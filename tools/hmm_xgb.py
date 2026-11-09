"""XGBoost HMM-regime state-classification experiments (hmm_ablation.md XGB-A..E).

Labels come from a causal, filtered Gaussian HMM that is a numpy port of
src/hmm.rs: same z-score scaler (train-slice only), same diagonal-Gaussian
Baum-Welch EM (random-observation mean init, uniform transition/initial,
transition pseudocount prior, covariance floor, relative-loglik tolerance,
best-of-restarts), and the same incremental filter_step used between refits.
The HMM is fit ONCE on the XGB train slice and frozen; validation/test labels
are produced by continuing the causal filter, never by smoothing or refitting.
State IDs are remapped once by fitted mean return (ascending) so class indices
are stable and ordered bear->bull.

Splits are chronological: train = dev minus last 20%, val = last 20% of dev
(early stopping), test = rows >= PARTITION_SPLIT (matches the study config).
Target is the NEXT day's state: features at row t predict argmax filtered
state at row t+1. Persistence baseline predicts S_{t+1} = S_t.

Run:  C:\\data\\envs\\trading_framework\\Scripts\\python.exe tools\\hmm_xgb.py
Outputs land in results/hmm_regime/ (xgb_summary.csv, xgb_confusion_<exp>.csv,
xgb_labels_k<K>.csv, xgb_report.md).
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

import numpy as np
import pandas as pd
from scipy.special import logsumexp
from sklearn.metrics import (
    balanced_accuracy_score,
    confusion_matrix,
    f1_score,
    log_loss,
)
from xgboost import XGBClassifier

DATA_DIR = Path(r"C:\data\trading\trading_framework\data\cache\etf_tr")
OUT_DIR = Path(__file__).resolve().parent.parent / "results" / "hmm_regime"
SYMBOL = "QQQ"
REF = "SPY"
PARTITION_SPLIT = "2024-01-01"
VAL_FRAC = 0.20  # last 20% of dev rows = chronological validation slice
SEED = 20261106
EPS = 1e-12

# HMM hyperparameters mirror configs/hmm_ablation.yaml baseline.
HMM = dict(
    max_iter=500, tol=1e-6, min_iter=5, restarts=4,
    cov_floor=1e-6, trans_prior=1.0,
)
RV_W, ATR_W, VOLCHG_W, SKEW_W, MA_F, MA_S, WILDER_W = 20, 14, 5, 20, 10, 50, 14

LABEL_FEATURES = ["ret", "rv", "atr", "volchg", "skew", "ma"]
EXPERIMENTS = {
    "XGB-A": dict(classes=5, feats=["ret", "rv", "atr", "volchg", "skew", "ma"]),
    "XGB-B": dict(classes=5, feats=["ret", "rv", "atr", "ma"]),
    "XGB-C": dict(classes=5, feats=["ret", "rv", "atr", "ma", "spy_ret"]),
    "XGB-D": dict(classes=5, feats=["ret", "rv", "atr", "ma", "wilder_atr", "spy_ret"]),
    "XGB-E": dict(classes=7, feats=["ret", "rv", "atr", "ma", "wilder_atr", "spy_ret"]),
}


# ---------------- features (port of src/hmm.rs feature_column) ----------------

def log_ret(x: pd.Series, w: int) -> pd.Series:
    out = pd.Series(np.nan, index=x.index)
    a, b = x.shift(w), x
    ok = a.gt(0) & b.gt(0)
    out[ok] = np.log(b[ok] / a[ok])
    return out


def rolling_std(x: pd.Series, w: int) -> pd.Series:
    return x.rolling(w).std(ddof=1).where(x.rolling(w).apply(lambda v: np.isfinite(v).all(), raw=True).astype(bool))


def true_range(c: pd.Series, h: pd.Series, l: pd.Series) -> pd.Series:
    tr = h - l
    pc = c.shift(1)
    tr = np.maximum(tr, np.maximum((h - pc).abs(), (l - pc).abs()))
    tr[(~np.isfinite(h)) | (~np.isfinite(l)) | (h < l)] = np.nan
    return tr


def wilder_atr(tr: pd.Series, w: int) -> pd.Series:
    out = np.full(len(tr), np.nan)
    v = tr.to_numpy()
    prev = np.nan
    for t in range(len(v)):
        if np.isfinite(prev) and np.isfinite(v[t]):
            prev = (prev * (w - 1) + v[t]) / w
        elif t + 1 >= w and np.isfinite(v[t + 1 - w : t + 1]).all():
            prev = v[t + 1 - w : t + 1].mean()
        out[t] = prev
    return pd.Series(out, index=tr.index)


def build_features(df: pd.DataFrame, ref_close: pd.Series) -> pd.DataFrame:
    c, h, l, v = df["close"], df["high"], df["low"], df["volume"].astype(float)
    ret = log_ret(c, 1)
    tr = true_range(c, h, l)
    feats = pd.DataFrame(index=df.index)
    feats["ret"] = ret
    feats["rv"] = rolling_std(ret, RV_W)
    feats["atr"] = np.log(tr.rolling(ATR_W).mean() / c)
    feats["volchg"] = log_ret(v, VOLCHG_W)
    m = ret.rolling(SKEW_W).mean()
    m2 = ret.rolling(SKEW_W).apply(lambda x: ((x - x.mean()) ** 2).mean(), raw=True)
    m3 = ret.rolling(SKEW_W).apply(lambda x: ((x - x.mean()) ** 3).mean(), raw=True)
    feats["skew"] = m3 / (np.sqrt(m2) ** 3 + EPS)
    feats["ma"] = c.rolling(MA_F).mean() / c.rolling(MA_S).mean() - 1.0
    feats["spy_ret"] = log_ret(ref_close.reindex(df.index), 1)
    feats["wilder_atr"] = np.log(wilder_atr(tr, WILDER_W) / c)
    return feats


# ---------------- Gaussian HMM (port of src/hmm.rs) ----------------

def log_gaussian_diag(x: np.ndarray, means: np.ndarray, var: np.ndarray) -> np.ndarray:
    d = x.shape[1]
    log_det = np.log(var).sum(axis=1)
    diff = x[:, None, :] - means[None, :, :]
    maha = (diff**2 / var[None, :, :]).sum(axis=2)
    return -0.5 * (d * np.log(2 * np.pi) + log_det[None, :] + maha)


def forward_backward(log_b: np.ndarray, trans: np.ndarray, init: np.ndarray):
    t, k = log_b.shape
    log_p = np.log(np.maximum(trans, EPS))
    log_pi = np.log(np.maximum(init, EPS))
    log_alpha = np.empty((t, k))
    log_scales = np.empty(t)
    raw = log_pi + log_b[0]
    log_scales[0] = logsumexp(raw)
    log_alpha[0] = raw - log_scales[0]
    for i in range(1, t):
        pred = logsumexp(log_alpha[i - 1][:, None] + log_p, axis=0)
        raw = log_b[i] + pred
        log_scales[i] = logsumexp(raw)
        log_alpha[i] = raw - log_scales[i]
    loglik = log_scales.sum()

    log_beta = np.zeros((t, k))
    for i in range(t - 2, -1, -1):
        log_beta[i] = logsumexp(
            log_p + log_b[i + 1][None, :] + log_beta[i + 1][None, :], axis=1
        ) - log_scales[i + 1]

    filtered = np.exp(log_alpha)
    lg = log_alpha + log_beta
    smoothed = np.exp(lg - logsumexp(lg, axis=1, keepdims=True))

    xi_sum = np.zeros((k, k))
    for i in range(t - 1):
        log_xi = (
            log_alpha[i][:, None]
            + log_p
            + log_b[i + 1][None, :]
            + log_beta[i + 1][None, :]
            - log_scales[i + 1]
        )
        xi_sum += np.exp(log_xi - logsumexp(log_xi))
    return filtered, smoothed, xi_sum, loglik


def em_once(x: np.ndarray, k: int, hp: dict, rng: np.random.Generator):
    t, d = x.shape
    idx = rng.permutation(t)[:k]
    means = x[idx].copy()
    variances = np.tile(np.maximum(x.var(axis=0), hp["cov_floor"]), (k, 1))
    trans = np.full((k, k), 1.0 / k)
    init = np.full(k, 1.0 / k)
    prev, loglik, n_iter, converged = -np.inf, -np.inf, 0, False
    filtered = np.tile(1.0 / k, (1, k))
    for it in range(1, hp["max_iter"] + 1):
        log_b = log_gaussian_diag(x, means, variances)
        filtered, smoothed, xi_sum, ll = forward_backward(log_b, trans, init)
        init = smoothed[0] + EPS
        init /= init.sum()
        trans = xi_sum + hp["trans_prior"]
        trans /= trans.sum(axis=1, keepdims=True)
        occ = smoothed.sum(axis=0)
        denom = np.maximum(occ, EPS)
        means = (smoothed.T @ x) / denom[:, None]
        diff = x[:, None, :] - means[None, :, :]
        variances = np.maximum(
            np.einsum("tk,tkd->kd", smoothed, diff**2) / denom[:, None],
            hp["cov_floor"],
        )
        loglik, n_iter = ll, it
        if it >= hp["min_iter"] and np.isfinite(prev) and abs(ll - prev) <= hp["tol"] * (1 + abs(prev)):
            converged = True
            break
        prev = ll
    model = dict(
        means=means, variances=variances, transition=trans, initial=init,
        loglik=loglik, n_iter=n_iter, converged=converged,
        last_filtered=filtered[-1].copy(),
    )
    return model, filtered


def fit_hmm(x: np.ndarray, k: int, hp: dict, seed: int):
    rng = np.random.default_rng(seed)
    best = None
    for _ in range(hp["restarts"]):
        m, f = em_once(x, k, hp, rng)
        if best is None or m["loglik"] > best[0]["loglik"]:
            best = (m, f)
    return best


def filter_forward(model: dict, x: np.ndarray) -> np.ndarray:
    """Causal filter chain from model['last_filtered'] over new rows x."""
    k = model["transition"].shape[0]
    prev = model["last_filtered"]
    out = np.empty((len(x), k))
    for t in range(len(x)):
        pred = prev @ model["transition"]
        log_b = -0.5 * (
            x.shape[1] * np.log(2 * np.pi)
            + np.log(model["variances"]).sum(axis=1)
            + ((x[t] - model["means"]) ** 2 / model["variances"]).sum(axis=1)
        )
        lj = np.log(np.maximum(pred, EPS)) + log_b
        out[t] = np.exp(lj - logsumexp(lj))
        prev = out[t]
    return out


def order_states(filtered: np.ndarray, rets: np.ndarray) -> np.ndarray:
    """State order by filtered-mass-weighted mean return, ascending."""
    k = filtered.shape[1]
    mass = np.maximum(filtered.sum(axis=0), EPS)
    mean = (filtered * rets[:, None]).sum(axis=0) / mass
    return np.argsort(mean)  # rank[i] position of state i


# ---------------- experiment driver ----------------

def causal_labels(k: int, x_train: np.ndarray, x_rest: np.ndarray,
                  train_rets: np.ndarray):
    """Fit on train, freeze, filter forward. Returns (ordered_labels_all, model)."""
    model, filt_train = fit_hmm(x_train, k, HMM, SEED)
    filt_rest = filter_forward(model, x_rest)
    order = order_states(filt_train, train_rets)
    remap = {s: int(r) for r, s in enumerate(order)}  # state -> class 0..k-1
    lab_train = np.array([remap[i] for i in filt_train.argmax(axis=1)])
    lab_rest = np.array([remap[i] for i in filt_rest.argmax(axis=1)])
    return np.concatenate([lab_train, lab_rest]), model, filt_train, filt_rest


def metrics(y_true, y_pred, y_proba, k):
    return dict(
        macro_f1=f1_score(y_true, y_pred, average="macro", zero_division=0),
        bal_acc=balanced_accuracy_score(y_true, y_pred),
        logloss=log_loss(y_true, y_proba, labels=list(range(k))),
        per_class_recall=confusion_matrix(
            y_true, y_pred, labels=list(range(k)), normalize="true"
        ).diagonal().round(3).tolist(),
    )


def main() -> int:
    t0 = time.time()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    df = pd.read_parquet(DATA_DIR / f"{SYMBOL}.parquet")
    ref = pd.read_parquet(DATA_DIR / f"{REF}.parquet")["close"]
    df.index = pd.to_datetime(df.index).tz_localize(None)
    ref.index = pd.to_datetime(ref.index).tz_localize(None)
    feats_all = build_features(df, ref)

    # Compact rows on the union of needed columns (label + experiment sets).
    needed = sorted({c for e in EXPERIMENTS.values() for c in e["feats"]}
                    | set(LABEL_FEATURES))
    mask = feats_all[needed].apply(np.isfinite).all(axis=1)
    frame = feats_all.loc[mask].copy()
    dates = frame.index
    x_lab = frame[LABEL_FEATURES].to_numpy()
    rets = frame["ret"].to_numpy()

    is_test = dates >= PARTITION_SPLIT
    dev_idx = np.where(~is_test)[0]
    n_val = int(round(len(dev_idx) * VAL_FRAC))
    tr_end = dev_idx[-1] - n_val + 1  # exclusive: train = rows[:tr_end]
    val_idx = dev_idx[-n_val:]
    te_idx = np.where(is_test)[0]
    print(f"rows={len(frame)} train={tr_end} val={len(val_idx)} test={len(te_idx)}")
    print(f"train {dates[0].date()}..{dates[tr_end-1].date()}  "
          f"val {dates[val_idx[0]].date()}..{dates[val_idx[-1]].date()}  "
          f"test {dates[te_idx[0]].date()}..{dates[te_idx[-1]].date()}")

    # --- label HMMs: fit on train slice only, frozen thereafter ---
    labels_by_k = {}
    for k in (5, 7):
        mu = x_lab[:tr_end].mean(axis=0)
        sd = x_lab[:tr_end].std(axis=0, ddof=1)
        z = (x_lab - mu) / sd
        lab, model, ftr, frest = causal_labels(k, z[:tr_end], z[tr_end:], rets[:tr_end])
        labels_by_k[k] = lab
        print(f"K={k}: loglik={model['loglik']:.1f} iters={model['n_iter']} "
              f"converged={model['converged']} P_ii={np.diag(model['transition']).round(3)}")
        lab_df = pd.DataFrame({"date": dates, "state": lab})
        lab_df.to_csv(OUT_DIR / f"xgb_labels_k{k}.csv", index=False)

    # --- XGB experiments ---
    rows, lines = [], []
    for name, spec in EXPERIMENTS.items():
        k, cols = spec["classes"], spec["feats"]
        lab = labels_by_k[k]
        x_all = frame[cols].to_numpy()
        # target: state at t+1 -> shift labels by one row; drop last row
        x, y, cur = x_all[:-1], lab[1:], lab[:-1]
        tr = np.arange(len(x)) < tr_end
        va = (np.arange(len(x)) >= tr_end) & (np.arange(len(x)) <= val_idx[-1])
        te = np.arange(len(x)) >= te_idx[0]

        clf = XGBClassifier(
            objective="multi:softprob", num_class=k, eval_metric="mlogloss",
            n_estimators=2000, max_depth=3, learning_rate=0.05,
            subsample=0.8, colsample_bytree=0.8,
            early_stopping_rounds=50, random_state=SEED, n_jobs=-1,
        )
        clf.fit(x[tr], y[tr], eval_set=[(x[va], y[va])], verbose=False)
        best = clf.best_iteration
        p_va, p_te = clf.predict(x[va]), clf.predict(x[te])
        prob_te = clf.predict_proba(x[te])
        # remap predict_proba columns to class ids if classes missing in train
        classes = clf.classes_.astype(int)
        full_prob = np.zeros((len(prob_te), k))
        full_prob[:, classes] = prob_te
        # persistence baseline: predict S_{t+1} = S_t
        pers_va, pers_te = cur[va], cur[te]
        pers_prob = np.eye(k)

        for split, yt, yp, ypb in [
            ("val", y[va], p_va, pers_va),
            ("test", y[te], p_te, pers_te),
        ]:
            if split == "val":
                prob = clf.predict_proba(x[va])
                pf = np.zeros((len(prob), k)); pf[:, classes] = prob
            else:
                pf = full_prob
            m = metrics(yt, yp, pf, k)
            mb = metrics(yt, ypb, pers_prob[ypb], k)
            rows.append(dict(experiment=name, classes=k, split=split,
                             n=len(yt), best_iter=best, **{f"xgb_{kk}": v for kk, v in m.items()},
                             **{f"persist_{kk}": v for kk, v in mb.items()}))
        cm = confusion_matrix(y[te], p_te, labels=list(range(k)))
        pd.DataFrame(cm).to_csv(OUT_DIR / f"xgb_confusion_{name.lower()}.csv")
        te_m = [r for r in rows if r["experiment"] == name and r["split"] == "test"][0]
        lines.append(
            f"| {name} | {k} | {len(cols)} | {best} | {te_m['xgb_macro_f1']:.3f} | "
            f"{te_m['xgb_bal_acc']:.3f} | {te_m['xgb_logloss']:.3f} | "
            f"{te_m['persist_macro_f1']:.3f} | {te_m['persist_bal_acc']:.3f} |"
        )
        print(f"{name}: test macroF1={te_m['xgb_macro_f1']:.3f} "
              f"balAcc={te_m['xgb_bal_acc']:.3f} vs persist "
              f"{te_m['persist_macro_f1']:.3f}/{te_m['persist_bal_acc']:.3f} "
              f"(best_iter={best})")

    pd.DataFrame(rows).to_csv(OUT_DIR / "xgb_summary.csv", index=False)
    report = (
        "## XGBoost state-classification results (test split)\n\n"
        "Labels: frozen causal filtered HMM fit on train slice only; "
        "state IDs ordered by mean return. Persistence = predict S(t+1)=S(t).\n\n"
        "| Exp | K | #feats | best_iter | XGB macroF1 | XGB balAcc | XGB logloss | "
        "persist macroF1 | persist balAcc |\n|---|---|---|---|---|---|---|---|---|\n"
        + "\n".join(lines) + "\n"
    )
    (OUT_DIR / "xgb_report.md").write_text(report)
    print(f"\n{time.time()-t0:.0f}s  -> {OUT_DIR}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
