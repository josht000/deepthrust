"""
markov_regime_models.py
=======================

Research-oriented reference implementations for four financial regime models:

1. GaussianHiddenMarkovModel
   Gaussian-emission HMM with constant transition probabilities. Includes
   Baum-Welch / EM fitting, forward filtering, forward-backward smoothing,
   Viterbi decoding, and multi-step regime-probability forecasts.

2. MarkovSwitchingAR
   Regime-dependent autoregressive return model. Conditional on observed
   lagged returns, each hidden state has its own intercept, AR coefficients,
   and innovation variance. Estimation is a weighted-regression EM algorithm.

3. MarkovSwitchingGARCH
   A clearly identified parallel-filter approximation to Markov-switching
   GARCH. Each regime has its own GARCH(1,1) variance recursion, and the
   regime probabilities are filtered through a Markov chain. This is NOT an
   exact path-dependent switching-GARCH likelihood. See the class docstring.

4. TimeVaryingTransitionHMM
   Gaussian-emission HMM whose transition probabilities depend on observed
   covariates through origin-state-specific multinomial logistic regressions.
   The transition model is estimated by generalized EM.

The module intentionally avoids downloading data, selecting a ticker,
choosing a trading frequency, or silently choosing feature thresholds.
Those are research decisions and must be supplied by the caller.

Dependencies:
    numpy
    scipy

Optional:
    pandas (only needed for the example feature-engineering helper)

Numerical conventions:
    - P[i, j] means P(S_t=j | S_{t-1}=i).
    - Rows of transition matrices sum to one.
    - Emission arrays are indexed [time, state].
    - Filtering probabilities are P(S_t | X_1, ..., X_t).
    - Smoothed probabilities are P(S_t | X_1, ..., X_T).
    - The first observation is conditioned on an explicit initial-state
      distribution. The likelihood convention is therefore conditional on
      the first feature vector under that initial distribution.
    - State labels are arbitrary. Never assume state 0 means Bull, etc.

This is reference/research code, not production trading software. In
particular, it does not provide exchange calendars, corporate-action
adjustments, survivorship-bias controls, execution simulation, or a complete
portfolio backtester.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

import numpy as np
from numpy.typing import ArrayLike, NDArray
from scipy.optimize import minimize
from scipy.special import logsumexp


FloatArray = NDArray[np.float64]
IntArray = NDArray[np.int64]


# ---------------------------------------------------------------------------
# Shared validation and numerical utilities
# ---------------------------------------------------------------------------

_EPS = 1e-12
_LOG_2PI = float(np.log(2.0 * np.pi))


def _as_2d_float(x: ArrayLike, name: str) -> FloatArray:
    """Convert input to a finite float64 matrix [time, feature]."""
    a = np.asarray(x, dtype=np.float64)
    if a.ndim == 1:
        a = a[:, None]
    if a.ndim != 2:
        raise ValueError(f"{name} must be one- or two-dimensional.")
    if a.shape[0] == 0 or a.shape[1] == 0:
        raise ValueError(f"{name} cannot have an empty time or feature axis.")
    if not np.isfinite(a).all():
        raise ValueError(
            f"{name} contains NaN or infinity. Decide how missing values "
            "should be handled before fitting; do not silently replace them."
        )
    return a


def _as_1d_float(x: ArrayLike, name: str) -> FloatArray:
    a = np.asarray(x, dtype=np.float64)
    if a.ndim != 1 or a.size == 0:
        raise ValueError(f"{name} must be a non-empty one-dimensional array.")
    if not np.isfinite(a).all():
        raise ValueError(f"{name} contains NaN or infinity.")
    return a


def _normalize_probabilities(p: ArrayLike, name: str) -> FloatArray:
    """Validate and normalize a probability vector."""
    a = np.asarray(p, dtype=np.float64)
    if a.ndim != 1 or np.any(a < 0.0) or not np.isfinite(a).all():
        raise ValueError(f"{name} must be a finite nonnegative vector.")
    total = float(a.sum())
    if total <= 0.0:
        raise ValueError(f"{name} must have positive sum.")
    return a / total


def _validate_transition_matrix(p: ArrayLike, n_states: int) -> FloatArray:
    """Validate a row-stochastic transition matrix."""
    a = np.asarray(p, dtype=np.float64)
    if a.shape != (n_states, n_states):
        raise ValueError(
            f"transition matrix must have shape {(n_states, n_states)}."
        )
    if not np.isfinite(a).all() or np.any(a < 0.0):
        raise ValueError("transition matrix must be finite and nonnegative.")
    row_sums = a.sum(axis=1)
    if np.any(row_sums <= 0.0):
        raise ValueError("every transition-matrix row needs positive mass.")
    return a / row_sums[:, None]


def _log_gaussian_diag(
    observations: FloatArray,
    means: FloatArray,
    variances: FloatArray,
) -> FloatArray:
    """
    Diagonal-covariance Gaussian log densities.

    observations: [T, D]
    means:        [K, D]
    variances:    [K, D]
    returns:      [T, K]

    A diagonal covariance is an explicit modeling choice: conditional feature
    independence is assumed within each state. It is not a full-covariance
    Gaussian model.
    """
    if np.any(variances <= 0.0):
        raise ValueError("Gaussian variances must be strictly positive.")
    diff = observations[:, None, :] - means[None, :, :]
    d = observations.shape[1]
    return -0.5 * (
        d * _LOG_2PI
        + np.log(variances).sum(axis=1)[None, :]
        + (diff * diff / variances[None, :, :]).sum(axis=2)
    )


def _forward_backward(
    log_emission: FloatArray,
    transition: FloatArray,
    initial: FloatArray,
) -> Tuple[FloatArray, FloatArray, FloatArray, float]:
    """
    Log-domain forward-backward for a time-homogeneous HMM.

    Returns:
        filtered: [T, K], P(S_t | observations through t)
        smoothed: [T, K], P(S_t | all observations)
        xi_sum:   [K, K], expected transition counts summed over t=1..T-1
        loglik:   scalar log likelihood

    This implementation is intentionally explicit because the distinction
    between filtered and smoothed probabilities is critical in trading.
    """
    t_count, k_count = log_emission.shape
    log_p = np.log(np.maximum(transition, _EPS))
    log_pi = np.log(np.maximum(initial, _EPS))

    log_alpha = np.empty((t_count, k_count), dtype=np.float64)
    log_alpha[0] = log_pi + log_emission[0]
    log_alpha[0] -= logsumexp(log_alpha[0])

    # Forward recursion. At each time, normalize to avoid underflow.
    log_scales = np.empty(t_count, dtype=np.float64)
    raw0 = log_pi + log_emission[0]
    log_scales[0] = logsumexp(raw0)
    log_alpha[0] = raw0 - log_scales[0]

    for t in range(1, t_count):
        raw = (
            log_emission[t]
            + logsumexp(log_alpha[t - 1][:, None] + log_p, axis=0)
        )
        log_scales[t] = logsumexp(raw)
        log_alpha[t] = raw - log_scales[t]

    loglik = float(log_scales.sum())

    # Backward recursion. Scaling is consistent with normalized forward rows.
    log_beta = np.zeros((t_count, k_count), dtype=np.float64)
    for t in range(t_count - 2, -1, -1):
        log_beta[t] = logsumexp(
            log_p + log_emission[t + 1][None, :] + log_beta[t + 1][None, :],
            axis=1,
        ) - log_scales[t + 1]

    log_gamma = log_alpha + log_beta
    log_gamma -= logsumexp(log_gamma, axis=1, keepdims=True)
    smoothed = np.exp(log_gamma)
    filtered = np.exp(log_alpha)

    xi_sum = np.zeros((k_count, k_count), dtype=np.float64)
    for t in range(t_count - 1):
        log_xi = (
            log_alpha[t][:, None]
            + log_p
            + log_emission[t + 1][None, :]
            + log_beta[t + 1][None, :]
            - log_scales[t + 1]
        )
        log_xi -= logsumexp(log_xi)
        xi_sum += np.exp(log_xi)

    return filtered, smoothed, xi_sum, loglik


def _stationary_distribution(transition: FloatArray) -> FloatArray:
    """
    Solve pi = pi P using the eigenvector nearest eigenvalue one.

    For reducible or periodic chains the stationary distribution may not be
    unique or convergence from arbitrary initial states may not occur. The
    returned vector is a numerical stationary solution, not a guarantee of
    ergodicity.
    """
    values, vectors = np.linalg.eig(transition.T)
    idx = int(np.argmin(np.abs(values - 1.0)))
    vector = np.real(vectors[:, idx])
    if vector.sum() < 0:
        vector = -vector
    vector = np.maximum(vector, 0.0)
    if vector.sum() <= _EPS:
        vector = np.abs(np.real(vectors[:, idx]))
    return vector / vector.sum()


def _make_lags(y: FloatArray, order: int) -> Tuple[FloatArray, FloatArray]:
    """Create [intercept, y(t-1), ..., y(t-p)] and aligned y(t)."""
    if order < 0:
        raise ValueError("AR order must be nonnegative.")
    if y.size <= order:
        raise ValueError("not enough observations for requested AR order.")
    design = np.ones((y.size - order, order + 1), dtype=np.float64)
    for lag in range(1, order + 1):
        design[:, lag] = y[order - lag : y.size - lag]
    return design, y[order:]


def _softmax_rows(logits: FloatArray) -> FloatArray:
    shifted = logits - logsumexp(logits, axis=1, keepdims=True)
    return np.exp(shifted)


# ---------------------------------------------------------------------------
# Shared output structures
# ---------------------------------------------------------------------------

@dataclass
class FilterResult:
    """Causal state estimates and one-step-ahead quantities."""
    filtered_probabilities: FloatArray
    predicted_probabilities: FloatArray
    log_likelihood: float


@dataclass
class ModelSelectionResult:
    """Information criteria computed from an in-sample log likelihood."""
    log_likelihood: float
    parameter_count: int
    sample_count: int
    aic: float
    bic: float


def information_criteria(loglik: float, parameter_count: int, n: int) -> ModelSelectionResult:
    """
    Calculate AIC and BIC.

    The caller must supply the parameter count appropriate to the fitted
    parameterization. Information criteria are only comparable when models
    use the same observations and likelihood convention.
    """
    if parameter_count < 0 or n <= 0:
        raise ValueError("parameter_count must be >= 0 and n must be > 0.")
    aic = 2.0 * parameter_count - 2.0 * loglik
    bic = np.log(n) * parameter_count - 2.0 * loglik
    return ModelSelectionResult(loglik, parameter_count, n, aic, bic)


# ---------------------------------------------------------------------------
# 1. Gaussian Hidden Markov Model
# ---------------------------------------------------------------------------

class GaussianHiddenMarkovModel:
    """
    Gaussian-emission HMM with diagonal covariance matrices.

    Generative model:
        S_0 ~ Categorical(initial_probabilities)
        S_t | S_{t-1}=i ~ Categorical(P[i, :])
        X_t | S_t=k ~ Normal(mean[k, :], diag(variance[k, :]))

    Fitting:
        Baum-Welch / EM:
          E-step: forward-backward posterior state and transition weights.
          M-step: update initial probabilities, transition matrix, means,
                  and diagonal variances using posterior weights.

    Practical notes:
        * EM can converge to local optima. Use multiple initializations.
        * State labels are permutation-invariant.
        * Feature scaling is important because Gaussian likelihoods are
          sensitive to feature units. Fit scalers on training data only.
        * Diagonal covariance is not equivalent to independent observations
          across time; the Markov chain still models temporal state dependence.
        * This class does not automatically select the number of states.
    """

    def __init__(
        self,
        n_states: int = 3,
        covariance_floor: float = 1e-6,
        transition_prior: float = 1.0,
        random_state: Optional[int] = None,
    ) -> None:
        if n_states < 1:
            raise ValueError("n_states must be at least one.")
        if covariance_floor <= 0.0:
            raise ValueError("covariance_floor must be positive.")
        if transition_prior < 0.0:
            raise ValueError("transition_prior must be nonnegative.")
        self.n_states = n_states
        self.covariance_floor = covariance_floor
        self.transition_prior = transition_prior
        self.rng = np.random.default_rng(random_state)
        self.means_: Optional[FloatArray] = None
        self.variances_: Optional[FloatArray] = None
        self.transition_: Optional[FloatArray] = None
        self.initial_: Optional[FloatArray] = None
        self.log_likelihood_: Optional[float] = None
        self.n_iter_: int = 0
        self.converged_: bool = False
        self.n_features_: Optional[int] = None

    def _initialize(self, x: FloatArray) -> None:
        t_count, d_count = x.shape
        if t_count < self.n_states:
            raise ValueError("number of observations must be >= n_states.")
        # Random observation centers are a simple initialization, not a
        # claim that k-means initialization is universally optimal.
        indices = self.rng.choice(t_count, self.n_states, replace=False)
        self.means_ = x[indices].copy()
        global_var = np.maximum(x.var(axis=0), self.covariance_floor)
        self.variances_ = np.tile(global_var, (self.n_states, 1))
        self.transition_ = np.full(
            (self.n_states, self.n_states), 1.0 / self.n_states
        )
        self.initial_ = np.full(self.n_states, 1.0 / self.n_states)
        self.n_features_ = d_count

    def _check_fitted(self) -> None:
        if any(v is None for v in (
            self.means_, self.variances_, self.transition_, self.initial_
        )):
            raise RuntimeError("model has not been fitted.")

    def fit(
        self,
        observations: ArrayLike,
        max_iter: int = 500,
        tolerance: float = 1e-6,
        minimum_iterations: int = 5,
    ) -> "GaussianHiddenMarkovModel":
        """Fit model parameters by Baum-Welch EM."""
        x = _as_2d_float(observations, "observations")
        if max_iter <= 0 or tolerance <= 0:
            raise ValueError("max_iter and tolerance must be positive.")
        self._initialize(x)
        previous = -np.inf

        for iteration in range(1, max_iter + 1):
            log_b = _log_gaussian_diag(x, self.means_, self.variances_)
            filtered, smoothed, xi_sum, loglik = _forward_backward(
                log_b, self.transition_, self.initial_
            )

            # M-step: posterior expected occupancy of each state.
            occupancy = smoothed.sum(axis=0)
            self.initial_ = _normalize_probabilities(
                smoothed[0] + _EPS, "updated initial probabilities"
            )

            # Transition prior is a pseudocount. A value of 1.0 corresponds
            # to a Laplace-style pseudocount; use 0.0 for unregularized counts.
            counts = xi_sum + self.transition_prior
            row_sums = counts.sum(axis=1, keepdims=True)
            self.transition_ = counts / np.maximum(row_sums, _EPS)

            for state in range(self.n_states):
                weights = smoothed[:, state]
                denom = max(float(occupancy[state]), _EPS)
                self.means_[state] = (weights[:, None] * x).sum(axis=0) / denom
                diff = x - self.means_[state]
                variance = (weights[:, None] * diff * diff).sum(axis=0) / denom
                self.variances_[state] = np.maximum(
                    variance, self.covariance_floor
                )

            self.log_likelihood_ = loglik
            self.n_iter_ = iteration
            if iteration >= minimum_iterations and np.isfinite(previous):
                if abs(loglik - previous) <= tolerance * (1.0 + abs(previous)):
                    self.converged_ = True
                    break
            previous = loglik

        return self

    def emission_log_likelihood(self, observations: ArrayLike) -> FloatArray:
        self._check_fitted()
        x = _as_2d_float(observations, "observations")
        if x.shape[1] != self.n_features_:
            raise ValueError("feature count differs from fitted model.")
        return _log_gaussian_diag(x, self.means_, self.variances_)

    def filter(self, observations: ArrayLike) -> FilterResult:
        """
        Causal filtering.

        predicted[t] = P(S_t | X_0, ..., X_{t-1})
        filtered[t]  = P(S_t | X_0, ..., X_t)

        The output is suitable for historical live-signal reconstruction.
        """
        self._check_fitted()
        x = _as_2d_float(observations, "observations")
        log_b = self.emission_log_likelihood(x)
        t_count = x.shape[0]
        filtered = np.empty((t_count, self.n_states))
        predicted = np.empty_like(filtered)

        predicted[0] = self.initial_
        log_joint = np.log(np.maximum(predicted[0], _EPS)) + log_b[0]
        filtered[0] = np.exp(log_joint - logsumexp(log_joint))
        loglik = float(logsumexp(log_joint))

        for t in range(1, t_count):
            predicted[t] = filtered[t - 1] @ self.transition_
            log_joint = np.log(np.maximum(predicted[t], _EPS)) + log_b[t]
            normalizer = logsumexp(log_joint)
            filtered[t] = np.exp(log_joint - normalizer)
            loglik += float(normalizer)

        return FilterResult(filtered, predicted, loglik)

    def smooth(self, observations: ArrayLike) -> FloatArray:
        """
        Return smoothed probabilities using the entire supplied sequence.

        WARNING: smoothed probabilities use future observations relative to
        each historical timestamp. They are useful for retrospective analysis
        and visualization, but are not valid as contemporaneous trading inputs.
        """
        self._check_fitted()
        x = _as_2d_float(observations, "observations")
        log_b = self.emission_log_likelihood(x)
        _, smoothed, _, _ = _forward_backward(
            log_b, self.transition_, self.initial_
        )
        return smoothed

    def viterbi(self, observations: ArrayLike) -> IntArray:
        """Return the single most likely complete hidden-state path."""
        self._check_fitted()
        x = _as_2d_float(observations, "observations")
        log_b = self.emission_log_likelihood(x)
        t_count, k_count = log_b.shape
        log_p = np.log(np.maximum(self.transition_, _EPS))
        score = np.empty((t_count, k_count))
        back = np.zeros((t_count, k_count), dtype=np.int64)
        score[0] = np.log(np.maximum(self.initial_, _EPS)) + log_b[0]

        for t in range(1, t_count):
            candidates = score[t - 1][:, None] + log_p
            back[t] = np.argmax(candidates, axis=0)
            score[t] = np.max(candidates, axis=0) + log_b[t]

        path = np.empty(t_count, dtype=np.int64)
        path[-1] = int(np.argmax(score[-1]))
        for t in range(t_count - 2, -1, -1):
            path[t] = back[t + 1, path[t + 1]]
        return path

    def forecast_regime_probabilities(
        self, last_filtered_probabilities: ArrayLike, horizon: int
    ) -> FloatArray:
        """
        Forecast regime probabilities without future observations.

        Output row h-1 is P(S_{t+h} | information through t).
        """
        self._check_fitted()
        if horizon < 1:
            raise ValueError("horizon must be at least one.")
        p = _normalize_probabilities(
            last_filtered_probabilities, "last_filtered_probabilities"
        )
        if p.size != self.n_states:
            raise ValueError("probability vector has wrong state count.")
        result = np.empty((horizon, self.n_states))
        for h in range(horizon):
            p = p @ self.transition_
            result[h] = p
        return result

    def stationary_distribution(self) -> FloatArray:
        self._check_fitted()
        return _stationary_distribution(self.transition_)

    def expected_state_duration(self) -> FloatArray:
        """
        Expected consecutive duration in each state for a homogeneous chain:
            E[D_i] = 1 / (1 - P[i, i]).
        A value is infinite when P[i, i] == 1.
        This is a geometric-duration assumption inherent to a first-order
        Markov chain; it is not a general duration model.
        """
        self._check_fitted()
        diagonal = np.diag(self.transition_)
        return np.where(
            diagonal >= 1.0 - _EPS, np.inf, 1.0 / (1.0 - diagonal)
        )


# ---------------------------------------------------------------------------
# 2. Markov-switching autoregression
# ---------------------------------------------------------------------------

class MarkovSwitchingAR:
    """
    Regime-dependent Gaussian autoregression.

    Conditional observation model:
        y_t | S_t=k, y_{t-1:t-p}
            ~ Normal(beta[k] @ [1, y_{t-1}, ..., y_{t-p}], variance[k])

        P(S_t=j | S_{t-1}=i) = transition[i, j]

    The lagged returns are observed regressors, not hidden random variables.
    This is a common conditional MS-AR specification. It is not the same as
    a model in which the AR recursion itself depends on an unobserved history
    of past regimes.

    Estimation:
        E-step: HMM forward-backward using state-specific regression residuals.
        M-step: weighted least squares for each state's coefficients and
                weighted residual variance; update transition probabilities.

    Important:
        The first `order` observations are consumed to construct lags and are
        not part of the conditional likelihood. Returned probabilities align
        with y[order:].
    """

    def __init__(
        self,
        n_states: int = 3,
        order: int = 1,
        variance_floor: float = 1e-10,
        transition_prior: float = 1.0,
        random_state: Optional[int] = None,
    ) -> None:
        if n_states < 1 or order < 0:
            raise ValueError("n_states >= 1 and order >= 0 are required.")
        self.n_states = n_states
        self.order = order
        self.variance_floor = variance_floor
        self.transition_prior = transition_prior
        self.rng = np.random.default_rng(random_state)
        self.coefficients_: Optional[FloatArray] = None  # [K, p+1]
        self.variances_: Optional[FloatArray] = None      # [K]
        self.transition_: Optional[FloatArray] = None
        self.initial_: Optional[FloatArray] = None
        self.log_likelihood_: Optional[float] = None
        self.converged_: bool = False
        self.n_iter_: int = 0

    def _emissions(self, design: FloatArray, target: FloatArray) -> FloatArray:
        means = design @ self.coefficients_.T
        residual = target[:, None] - means
        return -0.5 * (
            _LOG_2PI
            + np.log(self.variances_)[None, :]
            + residual * residual / self.variances_[None, :]
        )

    def fit(
        self,
        returns: ArrayLike,
        max_iter: int = 500,
        tolerance: float = 1e-6,
        minimum_iterations: int = 5,
    ) -> "MarkovSwitchingAR":
        y = _as_1d_float(returns, "returns")
        design, target = _make_lags(y, self.order)
        n, d = design.shape
        if n < self.n_states:
            raise ValueError("insufficient observations for number of states.")

        # Initialize state regressions from small random perturbations around
        # a pooled least-squares fit. Multiple random seeds should be tried.
        pooled = np.linalg.lstsq(design, target, rcond=None)[0]
        pooled_resid = target - design @ pooled
        pooled_var = max(float(np.var(pooled_resid)), self.variance_floor)
        self.coefficients_ = np.tile(pooled, (self.n_states, 1))
        self.coefficients_ += self.rng.normal(
            0.0, np.sqrt(pooled_var) * 0.01,
            size=(self.n_states, d)
        )
        self.variances_ = np.full(self.n_states, pooled_var)
        self.transition_ = np.full(
            (self.n_states, self.n_states), 1.0 / self.n_states
        )
        self.initial_ = np.full(self.n_states, 1.0 / self.n_states)

        previous = -np.inf
        for iteration in range(1, max_iter + 1):
            log_b = self._emissions(design, target)
            _, gamma, xi_sum, loglik = _forward_backward(
                log_b, self.transition_, self.initial_
            )

            occupancy = gamma.sum(axis=0)
            self.initial_ = _normalize_probabilities(gamma[0] + _EPS, "initial")
            counts = xi_sum + self.transition_prior
            self.transition_ = counts / np.maximum(
                counts.sum(axis=1, keepdims=True), _EPS
            )

            for state in range(self.n_states):
                w = gamma[:, state]
                # Weighted least squares solves:
                # argmin_beta sum_t gamma[t,k] * (y[t] - X[t] beta)^2.
                weighted_x = design * np.sqrt(w[:, None])
                weighted_y = target * np.sqrt(w)
                beta = np.linalg.lstsq(weighted_x, weighted_y, rcond=None)[0]
                self.coefficients_[state] = beta
                residual = target - design @ beta
                variance = np.sum(w * residual * residual) / max(
                    occupancy[state], _EPS
                )
                self.variances_[state] = max(
                    float(variance), self.variance_floor
                )

            self.log_likelihood_ = loglik
            self.n_iter_ = iteration
            if iteration >= minimum_iterations and np.isfinite(previous):
                if abs(loglik - previous) <= tolerance * (1.0 + abs(previous)):
                    self.converged_ = True
                    break
            previous = loglik

        return self

    def filter(self, returns: ArrayLike) -> FilterResult:
        if self.coefficients_ is None:
            raise RuntimeError("model has not been fitted.")
        y = _as_1d_float(returns, "returns")
        design, target = _make_lags(y, self.order)
        log_b = self._emissions(design, target)
        # Reuse the same causal recursion as the Gaussian HMM.
        t_count = len(target)
        filtered = np.empty((t_count, self.n_states))
        predicted = np.empty_like(filtered)
        predicted[0] = self.initial_
        joint = np.log(np.maximum(predicted[0], _EPS)) + log_b[0]
        z = logsumexp(joint)
        filtered[0] = np.exp(joint - z)
        loglik = float(z)
        for t in range(1, t_count):
            predicted[t] = filtered[t - 1] @ self.transition_
            joint = np.log(np.maximum(predicted[t], _EPS)) + log_b[t]
            z = logsumexp(joint)
            filtered[t] = np.exp(joint - z)
            loglik += float(z)
        return FilterResult(filtered, predicted, loglik)

    def forecast_next_return_distribution(
        self, last_returns: ArrayLike, state_probabilities: ArrayLike
    ) -> Dict[str, Any]:
        """
        One-step predictive mixture distribution.

        Returns state-conditional means/variances and mixture mean/variance.
        The supplied probabilities must represent P(S_{t+1} | information_t),
        not smoothed probabilities.
        """
        if self.coefficients_ is None:
            raise RuntimeError("model has not been fitted.")
        history = _as_1d_float(last_returns, "last_returns")
        if history.size < self.order:
            raise ValueError("last_returns does not contain enough lags.")
        p = _normalize_probabilities(state_probabilities, "state_probabilities")
        if p.size != self.n_states:
            raise ValueError("state probability vector has wrong length.")
        x = np.ones(self.order + 1)
        for lag in range(1, self.order + 1):
            x[lag] = history[-lag]
        means = self.coefficients_ @ x
        variances = self.variances_.copy()
        mixture_mean = float(p @ means)
        mixture_variance = float(p @ (variances + means * means) - mixture_mean**2)
        return {
            "state_probabilities": p,
            "state_conditional_means": means,
            "state_conditional_variances": variances,
            "mixture_mean": mixture_mean,
            "mixture_variance": max(mixture_variance, 0.0),
        }


# ---------------------------------------------------------------------------
# 3. Markov-switching GARCH(1,1), explicitly approximate
# ---------------------------------------------------------------------------

class MarkovSwitchingGARCH:
    """
    Regime-switching GARCH(1,1) using parallel regime-conditioned variance
    filters.

    State-conditional return model:
        r_t | S_t=k ~ Normal(mu_k, h[t,k])

    Parallel variance recursion:
        h[t,k] = omega_k
                 + alpha_k * (r[t-1] - mu_k)^2
                 + beta_k * h[t-1,k]

    State probabilities evolve through a Markov transition matrix and are
    updated using the state-specific Gaussian likelihoods.

    IMPORTANT MODEL LIMITATION:
        This is a tractable parallel-filter approximation. Each state's
        variance recursion is advanced using the observed previous return,
        even when the previous hidden state may have been different. It does
        not integrate over every possible regime path and is NOT an exact
        path-dependent Markov-switching GARCH likelihood.

    Why expose this approximation:
        Exact switching-GARCH formulations require a specific treatment of
        latent-state-dependent variance histories. Depending on the model,
        exact filtering can require a growing mixture of variance components.
        Published approximations (for example, Gray-style collapsing) make
        additional assumptions. Those assumptions must be selected and
        validated rather than hidden behind a generic "MS-GARCH" label.

    This implementation estimates parameters by direct numerical
    likelihood maximization with constraints:
        omega > 0
        alpha >= 0
        beta >= 0
        alpha + beta < persistence_limit < 1

    The persistence constraint is a modeling/numerical restriction, not a
    universal law for every GARCH specification.
    """

    def __init__(
        self,
        n_states: int = 3,
        persistence_limit: float = 0.999,
        variance_floor: float = 1e-10,
        random_state: Optional[int] = None,
    ) -> None:
        if n_states < 1:
            raise ValueError("n_states must be >= 1.")
        if not 0.0 < persistence_limit < 1.0:
            raise ValueError("persistence_limit must be between zero and one.")
        self.n_states = n_states
        self.persistence_limit = persistence_limit
        self.variance_floor = variance_floor
        self.rng = np.random.default_rng(random_state)
        self.means_: Optional[FloatArray] = None
        self.omega_: Optional[FloatArray] = None
        self.alpha_: Optional[FloatArray] = None
        self.beta_: Optional[FloatArray] = None
        self.transition_: Optional[FloatArray] = None
        self.initial_: Optional[FloatArray] = None
        self.initial_variance_: Optional[FloatArray] = None
        self.log_likelihood_: Optional[float] = None
        self.optimizer_result_: Optional[Any] = None

    def _unpack(self, theta: FloatArray) -> Tuple[FloatArray, ...]:
        k = self.n_states
        means = theta[0:k]
        omega = np.exp(theta[k:2*k])
        # Softmax-like bounded parameterization enforces alpha + beta < limit.
        a_raw = np.exp(theta[2*k:3*k])
        b_raw = np.exp(theta[3*k:4*k])
        denom = 1.0 + a_raw + b_raw
        alpha = self.persistence_limit * a_raw / denom
        beta = self.persistence_limit * b_raw / denom
        return means, omega, alpha, beta

    def _filter_with_parameters(
        self,
        returns: FloatArray,
        means: FloatArray,
        omega: FloatArray,
        alpha: FloatArray,
        beta: FloatArray,
        transition: FloatArray,
        initial: FloatArray,
        initial_variance: FloatArray,
    ) -> Tuple[FloatArray, FloatArray, float]:
        t_count = returns.size
        k = self.n_states
        h = np.empty((t_count, k))
        log_b = np.empty((t_count, k))
        filtered = np.empty((t_count, k))
        predicted = np.empty((t_count, k))

        h[0] = np.maximum(initial_variance, self.variance_floor)
        for t in range(t_count):
            if t > 0:
                residual_previous = returns[t - 1] - means
                h[t] = (
                    omega
                    + alpha * residual_previous**2
                    + beta * h[t - 1]
                )
                h[t] = np.maximum(h[t], self.variance_floor)

            residual = returns[t] - means
            log_b[t] = -0.5 * (
                _LOG_2PI + np.log(h[t]) + residual**2 / h[t]
            )

            if t == 0:
                predicted[t] = initial
            else:
                predicted[t] = filtered[t - 1] @ transition

            joint = np.log(np.maximum(predicted[t], _EPS)) + log_b[t]
            normalizer = logsumexp(joint)
            filtered[t] = np.exp(joint - normalizer)
            if t == 0:
                loglik = float(normalizer)
            else:
                loglik += float(normalizer)

        return filtered, h, loglik

    def fit(
        self,
        returns: ArrayLike,
        max_iterations: int = 2000,
        initial_transition: Optional[ArrayLike] = None,
    ) -> "MarkovSwitchingGARCH":
        y = _as_1d_float(returns, "returns")
        if y.size < max(30, self.n_states * 10):
            raise ValueError(
                "sample is short for this parameterization; supply a longer "
                "sample or explicitly revise this research safeguard."
            )
        k = self.n_states
        var = max(float(np.var(y)), self.variance_floor)
        self.transition_ = (
            _validate_transition_matrix(initial_transition, k)
            if initial_transition is not None
            else np.full((k, k), 1.0 / k)
        )
        self.initial_ = np.full(k, 1.0 / k)
        self.initial_variance_ = np.full(k, var)

        # Means initialize at empirical quantiles. This is only an optimizer
        # starting point; it does not define the final state labels.
        quantiles = np.linspace(0.2, 0.8, k)
        means0 = np.quantile(y, quantiles)
        omega0 = np.full(k, var * 0.05)
        alpha0 = np.full(k, 0.05)
        beta0 = np.full(k, 0.85)
        # Invert the bounded alpha/beta parameterization approximately.
        remaining = self.persistence_limit
        raw_a = alpha0 / np.maximum(remaining - alpha0 - beta0, 1e-4)
        raw_b = beta0 / np.maximum(remaining - alpha0 - beta0, 1e-4)
        theta0 = np.concatenate([
            means0,
            np.log(omega0),
            np.log(raw_a),
            np.log(raw_b),
        ])

        def objective(theta: FloatArray) -> float:
            means, omega, alpha, beta = self._unpack(theta)
            _, _, ll = self._filter_with_parameters(
                y, means, omega, alpha, beta,
                self.transition_, self.initial_, self.initial_variance_
            )
            return -ll if np.isfinite(ll) else 1e100

        result = minimize(
            objective,
            theta0,
            method="L-BFGS-B",
            options={"maxiter": max_iterations},
        )
        self.optimizer_result_ = result
        self.means_, self.omega_, self.alpha_, self.beta_ = self._unpack(result.x)
        filtered, h, ll = self._filter_with_parameters(
            y, self.means_, self.omega_, self.alpha_, self.beta_,
            self.transition_, self.initial_, self.initial_variance_
        )
        self.log_likelihood_ = ll
        self.filtered_probabilities_ = filtered
        self.conditional_variances_ = h
        return self

    def filter(self, returns: ArrayLike) -> FilterResult:
        if self.means_ is None:
            raise RuntimeError("model has not been fitted.")
        y = _as_1d_float(returns, "returns")
        filtered, h, ll = self._filter_with_parameters(
            y, self.means_, self.omega_, self.alpha_, self.beta_,
            self.transition_, self.initial_, self.initial_variance_
        )
        predicted = np.empty_like(filtered)
        predicted[0] = self.initial_
        for t in range(1, len(y)):
            predicted[t] = filtered[t - 1] @ self.transition_
        return FilterResult(filtered, predicted, ll)

    def forecast_next_variance(
        self,
        last_return: float,
        last_regime_probabilities: ArrayLike,
        last_regime_variances: ArrayLike,
    ) -> Dict[str, FloatArray]:
        """
        One-step variance forecast under the parallel-filter approximation.

        Inputs are the filtered regime probabilities and per-state conditional
        variances at time t. The returned regime probabilities are predicted
        for t+1. Per-state h[t+1,k] uses the latest observed return.

        The mixture variance includes both within-state variance and
        between-state mean dispersion.
        """
        if self.means_ is None:
            raise RuntimeError("model has not been fitted.")
        p = _normalize_probabilities(last_regime_probabilities, "probabilities")
        h_prev = _as_1d_float(last_regime_variances, "last_regime_variances")
        if p.size != self.n_states or h_prev.size != self.n_states:
            raise ValueError("state vectors must match n_states.")
        h_next = (
            self.omega_
            + self.alpha_ * (float(last_return) - self.means_)**2
            + self.beta_ * h_prev
        )
        h_next = np.maximum(h_next, self.variance_floor)
        p_next = p @ self.transition_
        mean_next = float(p_next @ self.means_)
        variance_next = float(
            p_next @ (h_next + self.means_**2) - mean_next**2
        )
        return {
            "predicted_regime_probabilities": p_next,
            "state_conditional_variances": h_next,
            "mixture_mean": np.array([mean_next]),
            "mixture_variance": np.array([max(variance_next, 0.0)]),
        }


# ---------------------------------------------------------------------------
# 4. Time-varying transition probability Gaussian HMM
# ---------------------------------------------------------------------------

class TimeVaryingTransitionHMM:
    """
    Gaussian-emission HMM with origin-state-specific multinomial-logit
    transition probabilities.

    Transition model:
        P(S_t=j | S_{t-1}=i, z_{t-1})
            = softmax_j( a[i,j] + B[i,j,:] @ z_{t-1} )

    z[t-1] must be known at the time the transition to t is forecast. Examples
    might include lagged realized volatility, lagged credit spreads, or
    lagged rates. Do not use contemporaneous or future values unless they
    would genuinely be observable at the forecast decision time.

    Estimation is generalized EM:
        E-step:
            Forward-backward with a different transition matrix at each t.
        M-step:
            Weighted Gaussian emission updates.
            For each origin state i, fit a multinomial logistic regression
            using expected transition counts xi[t,i,j] as fractional labels.

    Identifiability:
        The last destination class is used as the zero-logit reference class
        for each origin state. This removes the softmax additive-constant
        ambiguity.

    The covariates are standardized by the caller. This class deliberately
    does not fit a scaler, because a production walk-forward pipeline must
    fit preprocessing only on the training segment and reuse it unchanged.
    """

    def __init__(
        self,
        n_states: int = 3,
        covariance_floor: float = 1e-6,
        l2_penalty: float = 1e-4,
        random_state: Optional[int] = None,
    ) -> None:
        if n_states < 1 or l2_penalty < 0:
            raise ValueError("invalid n_states or l2_penalty.")
        self.n_states = n_states
        self.covariance_floor = covariance_floor
        self.l2_penalty = l2_penalty
        self.rng = np.random.default_rng(random_state)
        self.means_: Optional[FloatArray] = None
        self.variances_: Optional[FloatArray] = None
        self.initial_: Optional[FloatArray] = None
        self.intercepts_: Optional[FloatArray] = None  # [origin, destination]
        self.slopes_: Optional[FloatArray] = None      # [origin, destination, Z]
        self.n_features_: Optional[int] = None
        self.n_transition_features_: Optional[int] = None
        self.log_likelihood_: Optional[float] = None
        self.converged_: bool = False
        self.n_iter_: int = 0

    def _transition_at(self, z_row: FloatArray) -> FloatArray:
        # Last destination is reference category with logit zero.
        logits = self.intercepts_ + np.einsum("kdf,f->kd", self.slopes_, z_row)
        return _softmax_rows(logits)

    def _forward_backward_tvtp(
        self, log_b: FloatArray, z: FloatArray
    ) -> Tuple[FloatArray, FloatArray, FloatArray, float]:
        t_count, k = log_b.shape
        if z.shape[0] != t_count - 1:
            raise ValueError(
                "transition covariates must have T-1 rows, aligned so row t-1 "
                "predicts the transition into observation t."
            )

        transitions = np.empty((t_count - 1, k, k))
        for t in range(1, t_count):
            transitions[t - 1] = self._transition_at(z[t - 1])

        log_alpha = np.empty((t_count, k))
        log_scale = np.empty(t_count)
        raw = np.log(np.maximum(self.initial_, _EPS)) + log_b[0]
        log_scale[0] = logsumexp(raw)
        log_alpha[0] = raw - log_scale[0]

        for t in range(1, t_count):
            log_p = np.log(np.maximum(transitions[t - 1], _EPS))
            raw = log_b[t] + logsumexp(
                log_alpha[t - 1][:, None] + log_p, axis=0
            )
            log_scale[t] = logsumexp(raw)
            log_alpha[t] = raw - log_scale[t]

        log_beta = np.zeros((t_count, k))
        for t in range(t_count - 2, -1, -1):
            log_p = np.log(np.maximum(transitions[t], _EPS))
            log_beta[t] = logsumexp(
                log_p + log_b[t + 1][None, :] + log_beta[t + 1][None, :],
                axis=1,
            ) - log_scale[t + 1]

        log_gamma = log_alpha + log_beta
        log_gamma -= logsumexp(log_gamma, axis=1, keepdims=True)
        gamma = np.exp(log_gamma)

        xi = np.empty((t_count - 1, k, k))
        for t in range(t_count - 1):
            log_p = np.log(np.maximum(transitions[t], _EPS))
            lx = (
                log_alpha[t][:, None] + log_p
                + log_b[t + 1][None, :] + log_beta[t + 1][None, :]
                - log_scale[t + 1]
            )
            lx -= logsumexp(lx)
            xi[t] = np.exp(lx)

        return gamma, xi, transitions, float(log_scale.sum())

    def _fit_transition_logit(
        self, z: FloatArray, expected_counts: FloatArray
    ) -> Tuple[FloatArray, FloatArray]:
        """
        Fit one weighted multinomial logistic regression per origin state.

        expected_counts[t, i, j] is the posterior expected probability mass
        for transition i -> j at time t. It is a fractional response, not a
        hard state label. The negative expected complete-data log likelihood
        is optimized with L-BFGS-B.
        """
        k = self.n_states
        n, f = z.shape
        intercepts = np.zeros((k, k))
        slopes = np.zeros((k, k, f))
        design = np.column_stack([np.ones(n), z])

        for origin in range(k):
            weights = expected_counts[:, origin, :]
            # Parameterize only K-1 destination logits; final destination
            # has fixed logit zero for identifiability.
            def objective(flat: FloatArray) -> Tuple[float, FloatArray]:
                params = flat.reshape(k - 1, f + 1)
                logits_free = design @ params.T
                logits = np.column_stack([logits_free, np.zeros(n)])
                probabilities = _softmax_rows(logits)
                loss = -np.sum(weights * np.log(np.maximum(probabilities, _EPS)))
                loss += 0.5 * self.l2_penalty * np.sum(params[:, 1:]**2)

                residual = probabilities[:, :k-1] * weights.sum(axis=1)[:, None]
                residual -= weights[:, :k-1]
                grad = residual.T @ design
                grad[:, 1:] += self.l2_penalty * params[:, 1:]
                return float(loss), grad.ravel()

            initial = np.zeros((k - 1, f + 1), dtype=np.float64).ravel()
            result = minimize(
                objective, initial, jac=True, method="L-BFGS-B",
                options={"maxiter": 500}
            )
            params = result.x.reshape(k - 1, f + 1)
            intercepts[origin, :k-1] = params[:, 0]
            slopes[origin, :k-1, :] = params[:, 1:]

        return intercepts, slopes

    def fit(
        self,
        observations: ArrayLike,
        transition_covariates: ArrayLike,
        max_iter: int = 300,
        tolerance: float = 1e-6,
        minimum_iterations: int = 5,
    ) -> "TimeVaryingTransitionHMM":
        x = _as_2d_float(observations, "observations")
        z = _as_2d_float(transition_covariates, "transition_covariates")
        if z.shape[0] != x.shape[0] - 1:
            raise ValueError(
                "transition_covariates must have T-1 rows. Row t-1 predicts "
                "the transition into observation t."
            )
        if x.shape[0] < self.n_states:
            raise ValueError("insufficient observations for n_states.")

        t_count, d = x.shape
        self.n_features_ = d
        self.n_transition_features_ = z.shape[1]
        centers = self.rng.choice(t_count, self.n_states, replace=False)
        self.means_ = x[centers].copy()
        global_var = np.maximum(x.var(axis=0), self.covariance_floor)
        self.variances_ = np.tile(global_var, (self.n_states, 1))
        self.initial_ = np.full(self.n_states, 1.0 / self.n_states)
        self.intercepts_ = np.zeros((self.n_states, self.n_states))
        self.slopes_ = np.zeros(
            (self.n_states, self.n_states, z.shape[1])
        )

        previous = -np.inf
        for iteration in range(1, max_iter + 1):
            log_b = _log_gaussian_diag(x, self.means_, self.variances_)
            gamma, xi, transitions, loglik = self._forward_backward_tvtp(log_b, z)

            self.initial_ = _normalize_probabilities(gamma[0] + _EPS, "initial")
            self.intercepts_, self.slopes_ = self._fit_transition_logit(z, xi)

            occupancy = gamma.sum(axis=0)
            for state in range(self.n_states):
                w = gamma[:, state]
                denom = max(float(occupancy[state]), _EPS)
                self.means_[state] = (w[:, None] * x).sum(axis=0) / denom
                diff = x - self.means_[state]
                self.variances_[state] = np.maximum(
                    (w[:, None] * diff * diff).sum(axis=0) / denom,
                    self.covariance_floor,
                )

            self.log_likelihood_ = loglik
            self.n_iter_ = iteration
            if iteration >= minimum_iterations and np.isfinite(previous):
                if abs(loglik - previous) <= tolerance * (1.0 + abs(previous)):
                    self.converged_ = True
                    break
            previous = loglik

        return self

    def filter(
        self,
        observations: ArrayLike,
        transition_covariates: ArrayLike,
    ) -> FilterResult:
        """
        Causal filter for a sequence with T observations and T-1 transition
        covariate rows. Covariate row t-1 predicts state transition into t.
        """
        if self.means_ is None:
            raise RuntimeError("model has not been fitted.")
        x = _as_2d_float(observations, "observations")
        z = _as_2d_float(transition_covariates, "transition_covariates")
        if x.shape[0] != z.shape[0] + 1:
            raise ValueError("expected len(z) == len(x)-1.")
        log_b = _log_gaussian_diag(x, self.means_, self.variances_)
        t_count = x.shape[0]
        filtered = np.empty((t_count, self.n_states))
        predicted = np.empty_like(filtered)

        predicted[0] = self.initial_
        joint = np.log(np.maximum(predicted[0], _EPS)) + log_b[0]
        norm = logsumexp(joint)
        filtered[0] = np.exp(joint - norm)
        loglik = float(norm)

        for t in range(1, t_count):
            p_t = self._transition_at(z[t - 1])
            predicted[t] = filtered[t - 1] @ p_t
            joint = np.log(np.maximum(predicted[t], _EPS)) + log_b[t]
            norm = logsumexp(joint)
            filtered[t] = np.exp(joint - norm)
            loglik += float(norm)

        return FilterResult(filtered, predicted, loglik)

    def forecast_next_regime_probabilities(
        self,
        last_filtered_probabilities: ArrayLike,
        next_transition_covariates: ArrayLike,
    ) -> FloatArray:
        """
        Forecast P(S_{t+1} | information through t).

        `next_transition_covariates` must be known at forecast time and must
        be aligned to the transition t -> t+1.
        """
        if self.means_ is None:
            raise RuntimeError("model has not been fitted.")
        p = _normalize_probabilities(
            last_filtered_probabilities, "last_filtered_probabilities"
        )
        z = _as_1d_float(next_transition_covariates, "next_transition_covariates")
        if p.size != self.n_states or z.size != self.n_transition_features_:
            raise ValueError("wrong state-probability or covariate dimension.")
        return p @ self._transition_at(z)

    def transition_matrix_for_covariates(
        self, transition_covariates: ArrayLike
    ) -> FloatArray:
        """Return the conditional transition matrix for one covariate vector."""
        if self.means_ is None:
            raise RuntimeError("model has not been fitted.")
        z = _as_1d_float(transition_covariates, "transition_covariates")
        if z.size != self.n_transition_features_:
            raise ValueError("wrong transition-covariate dimension.")
        return self._transition_at(z)


# ---------------------------------------------------------------------------
# Feature engineering: deliberately explicit, with no implicit data policy
# ---------------------------------------------------------------------------

@dataclass
class FeatureSet:
    """
    Feature matrix plus metadata needed to interpret it.

    `values` is [time, feature]. `names` gives column names. `valid_index`
    preserves the original index when built from pandas.
    """
    values: FloatArray
    names: List[str]
    valid_index: Any = None


def build_market_features(
    close: ArrayLike,
    high: ArrayLike,
    low: ArrayLike,
    volume: ArrayLike,
    realized_volatility_window: int,
    atr_window: int,
    volume_change_window: int,
    skewness_window: int,
    moving_average_fast: int,
    moving_average_slow: int,
    annualization_factor: Optional[float] = None,
) -> FeatureSet:
    """
    Build a concrete feature matrix from OHLCV arrays.

    Features:
        1. daily_log_return
        2. rolling_realized_volatility
        3. normalized_atr
        4. log_volume_change
        5. rolling_return_skewness
        6. moving_average_spread

    Definitions:
        log return:
            log(close[t] / close[t-1])

        realized volatility:
            rolling sample standard deviation of daily log returns.
            If annualization_factor is supplied, multiply by its square root.
            The factor must reflect the data frequency and desired convention.

        ATR:
            Wilder-style smoothing is NOT used here. This helper uses a
            simple rolling mean of true range. If Wilder ATR is desired,
            implement that exact convention separately.

        normalized ATR:
            rolling mean true range / close

        volume change:
            log(volume[t] / volume[t-volume_change_window]).
            Requires strictly positive volume values.

        return skewness:
            rolling biased third standardized central moment.
            This is a descriptive feature, not an unbiased estimator.

        moving-average spread:
            SMA(fast) / SMA(slow) - 1

    No missing-value imputation is performed. The initial warm-up region is
    dropped, and any remaining invalid values cause an error. Corporate
    actions, split adjustment, bad ticks, session boundaries, and missing
    bars must be addressed before calling this function.
    """
    c = _as_1d_float(close, "close")
    h = _as_1d_float(high, "high")
    l = _as_1d_float(low, "low")
    v = _as_1d_float(volume, "volume")
    if not (len(c) == len(h) == len(l) == len(v)):
        raise ValueError("OHLCV arrays must have identical lengths.")
    windows = {
        "realized_volatility_window": realized_volatility_window,
        "atr_window": atr_window,
        "volume_change_window": volume_change_window,
        "skewness_window": skewness_window,
        "moving_average_fast": moving_average_fast,
        "moving_average_slow": moving_average_slow,
    }
    if any(value < 1 for value in windows.values()):
        raise ValueError("all feature windows must be positive integers.")
    if moving_average_fast >= moving_average_slow:
        raise ValueError("fast moving-average window must be smaller than slow.")
    if np.any(c <= 0) or np.any(v <= 0):
        raise ValueError("close and volume must be strictly positive.")
    if np.any(h < l):
        raise ValueError("high must be >= low.")

    log_return = np.full(len(c), np.nan)
    log_return[1:] = np.log(c[1:] / c[:-1])

    def rolling_mean(a: FloatArray, window: int) -> FloatArray:
        out = np.full(len(a), np.nan)
        for t in range(window - 1, len(a)):
            segment = a[t-window+1:t+1]
            if np.isfinite(segment).all():
                out[t] = segment.mean()
        return out

    def rolling_std(a: FloatArray, window: int) -> FloatArray:
        out = np.full(len(a), np.nan)
        for t in range(window - 1, len(a)):
            segment = a[t-window+1:t+1]
            if np.isfinite(segment).all():
                out[t] = segment.std(ddof=1) if window > 1 else 0.0
        return out

    rv = rolling_std(log_return, realized_volatility_window)
    if annualization_factor is not None:
        if annualization_factor <= 0:
            raise ValueError("annualization_factor must be positive.")
        rv *= np.sqrt(annualization_factor)

    previous_close = np.r_[np.nan, c[:-1]]
    true_range = np.maximum.reduce([
        h - l,
        np.abs(h - previous_close),
        np.abs(l - previous_close),
    ])
    atr = rolling_mean(true_range, atr_window) / c

    volume_change = np.full(len(v), np.nan)
    w = volume_change_window
    volume_change[w:] = np.log(v[w:] / v[:-w])

    skew = np.full(len(c), np.nan)
    for t in range(skewness_window - 1, len(c)):
        segment = log_return[t-skewness_window+1:t+1]
        if np.isfinite(segment).all():
            centered = segment - segment.mean()
            std = np.sqrt(np.mean(centered**2))
            skew[t] = np.mean(centered**3) / (std**3 + _EPS)

    fast = rolling_mean(c, moving_average_fast)
    slow = rolling_mean(c, moving_average_slow)
    ma_spread = fast / slow - 1.0

    matrix = np.column_stack([
        log_return, rv, atr, volume_change, skew, ma_spread
    ])
    names = [
        "daily_log_return",
        "rolling_realized_volatility",
        "normalized_atr",
        "log_volume_change",
        "rolling_return_skewness",
        "moving_average_spread",
    ]

    valid = np.isfinite(matrix).all(axis=1)
    if not valid.any():
        raise ValueError("no valid rows remain after feature warm-up.")
    return FeatureSet(matrix[valid], names, np.flatnonzero(valid))


# ---------------------------------------------------------------------------
# State interpretation and research diagnostics
# ---------------------------------------------------------------------------

def summarize_states(
    probabilities: ArrayLike,
    returns: ArrayLike,
    annualization_factor: Optional[float] = None,
) -> List[Dict[str, float]]:
    """
    Summarize states using posterior/filtered weights.

    This function does NOT automatically assign semantic labels. A state
    with positive mean return is not automatically a reliable Bull signal.
    It only reports weighted descriptive statistics.

    Use filtered probabilities for causal/live summaries. Smoothed
    probabilities are retrospective and can make historical regimes look
    cleaner than they would have appeared in real time.
    """
    p = _as_2d_float(probabilities, "probabilities")
    r = _as_1d_float(returns, "returns")
    if p.shape[0] != r.size:
        raise ValueError("probability and return lengths must match.")
    if np.any(p < 0) or not np.allclose(p.sum(axis=1), 1.0, atol=1e-6):
        raise ValueError("each probability row must be a probability vector.")

    summaries: List[Dict[str, float]] = []
    for state in range(p.shape[1]):
        w = p[:, state]
        mass = float(w.sum())
        mean = float(np.sum(w * r) / max(mass, _EPS))
        variance = float(np.sum(w * (r - mean)**2) / max(mass, _EPS))
        result = {
            "state": float(state),
            "effective_observations": mass,
            "mean_return_per_period": mean,
            "volatility_per_period": float(np.sqrt(max(variance, 0.0))),
        }
        if annualization_factor is not None:
            if annualization_factor <= 0:
                raise ValueError("annualization_factor must be positive.")
            result["annualized_mean_arithmetic_approx"] = (
                mean * annualization_factor
            )
            result["annualized_volatility"] = (
                np.sqrt(max(variance, 0.0)) * np.sqrt(annualization_factor)
            )
        summaries.append(result)
    return summaries


def order_states_by_return(
    state_summaries: Sequence[Dict[str, float]]
) -> List[int]:
    """
    Return state IDs sorted by weighted mean return, low to high.

    This is a convenience for interpretation only. It does not establish
    statistically stable Bull/Bear/Sideways labels. Consider volatility,
    duration, confidence intervals, and out-of-sample stability as well.
    """
    return [
        int(row["state"])
        for row in sorted(
            state_summaries,
            key=lambda item: item["mean_return_per_period"]
        )
    ]


# ---------------------------------------------------------------------------
# Walk-forward evaluation scaffolding
# ---------------------------------------------------------------------------

@dataclass
class WalkForwardFold:
    train_start: int
    train_end_exclusive: int
    test_start: int
    test_end_exclusive: int


def expanding_walk_forward_folds(
    n_observations: int,
    initial_train_size: int,
    test_size: int,
    step_size: Optional[int] = None,
) -> List[WalkForwardFold]:
    """
    Create expanding-window folds using integer positions.

    The caller must fit all preprocessing and model parameters using only
    [train_start, train_end_exclusive). The test segment must be evaluated
    without refitting on its observations unless the experiment explicitly
    specifies an online update protocol.

    This helper does not decide whether a gap/embargo is needed. For features
    with overlapping labels or forward-return targets, introduce an embargo
    appropriate to the target horizon.
    """
    if initial_train_size <= 0 or test_size <= 0:
        raise ValueError("initial_train_size and test_size must be positive.")
    step = test_size if step_size is None else step_size
    if step <= 0:
        raise ValueError("step_size must be positive.")
    folds: List[WalkForwardFold] = []
    test_start = initial_train_size
    while test_start + test_size <= n_observations:
        folds.append(WalkForwardFold(
            train_start=0,
            train_end_exclusive=test_start,
            test_start=test_start,
            test_end_exclusive=test_start + test_size,
        ))
        test_start += step
    return folds


def multiclass_log_loss(
    true_states: ArrayLike, probabilities: ArrayLike
) -> float:
    """
    Log loss for known state labels.

    In real financial regime discovery, true regimes are generally unknown.
    This metric is therefore only appropriate for synthetic experiments or
    externally labeled regimes. Do not treat a model's own Viterbi path as
    ground truth for evaluating that same model.
    """
    y = np.asarray(true_states, dtype=np.int64)
    p = _as_2d_float(probabilities, "probabilities")
    if y.ndim != 1 or y.size != p.shape[0]:
        raise ValueError("true_states must align with probability rows.")
    if np.any(y < 0) or np.any(y >= p.shape[1]):
        raise ValueError("true state index out of range.")
    return float(-np.mean(np.log(np.maximum(p[np.arange(y.size), y], _EPS))))


# ---------------------------------------------------------------------------
# Implementation and research checklist
# ---------------------------------------------------------------------------

RESEARCH_CHECKLIST = [
    "Define the traded instrument, market, bar frequency, session, and timezone.",
    "Use split/dividend-adjusted prices where appropriate; document the source.",
    "Define whether returns are simple or log returns and keep it consistent.",
    "Specify every feature formula, lookback, warm-up, and timestamp alignment.",
    "Fit feature scalers on training data only; never fit on the full dataset.",
    "Use multiple random initializations for EM models and retain diagnostics.",
    "Check state occupancy; reject solutions with effectively unused states.",
    "Inspect covariance/variance floors and whether estimates hit those floors.",
    "Compare state counts using information criteria on identical observations.",
    "Use filtered probabilities for signals; reserve smoothed probabilities for analysis.",
    "Evaluate forecast log likelihood on held-out data, not only training likelihood.",
    "Measure state persistence, transition frequency, and label stability.",
    "Test parameter sensitivity to sample window and market period.",
    "Account for commissions, spread, slippage, borrow costs, and market impact.",
    "Avoid tuning on the final test period; preserve a final untouched holdout.",
    "Compare against buy-and-hold, cash, simple momentum, and volatility baselines.",
    "Do not equate statistical regimes with profitable trading actions.",
    "Document the exact GARCH switching approximation used.",
]


if __name__ == "__main__":
    # This block is intentionally a structural example, not a complete
    # market-data experiment. No ticker, frequency, thresholds, or trading
    # rules are assumed.
    #
    # Replace `observations` and `returns` with correctly aligned data.
    # Standardization must be fitted on the training interval only.
    print("Reference module loaded.")
    print("Available models:")
    print(" - GaussianHiddenMarkovModel")
    print(" - MarkovSwitchingAR")
    print(" - MarkovSwitchingGARCH (parallel-filter approximation)")
    print(" - TimeVaryingTransitionHMM")
    print(f"Research checklist items: {len(RESEARCH_CHECKLIST)}")
