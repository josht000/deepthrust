# GitHub-Sourced Trading Strategy Catalog

Compiled from a code-level review of 216 in-scope repositories cloned from the
[GitHub Trading Algorithm Repository Inventory](GitHub%20Trading%20Algorithm%20Repository%20Inventory.md).
Companion machine-readable file: `GitHub Strategy Catalog (JSON for coding agent).json`.

**Focus:** strategies whose essence works on daily bars, overnight holds and short swings
(multiple trades/week potential). Intraday-native strategies are included but flagged in the
*Best timeframes* field. Every entry cites the exact code it was extracted from (permalinked at
clone SHA). Overlaps with the existing 105-strategy catalog are marked `overlap:` -> S###.
Priorities: **High** = daily-native + precise rules + real mechanism or lineage; **Medium** =
clear rules, weaker evidence or needs adaptation; **Low** = intraday-only / weak / anti-pattern.
Repo-claimed metrics are marked `(repo-claimed, unverified)` - nothing here was re-backtested.

**209 strategies** (9 High, 59 Medium, 141 Low)

## Table of Contents

- [Single-Asset Mean Reversion (oscillators, bands, z-score)](#single-asset-mean-reversion-oscillators-bands-z-score) (22)
  - [G001 | Daily-Checkpoint Oversold-Bounce with Trend-Midpoint Exits (Mag10 + index ETFs)](#g001--daily-checkpoint-oversold-bounce-with-trend-midpoint-exits-mag10--index-etfs)
  - [G002 | Bollinger(30,2) Band-Touch Mean Reversion with Neutral Zone (EUR/USD)](#g002--bollinger302-band-touch-mean-reversion-with-neutral-zone-eurusd)
  - [G003 | Daily High-Low Geometric-Midpoint Reversal](#g003--daily-high-low-geometric-midpoint-reversal)
  - [G004 | Double-Seven Connors Pullback](#g004--double-seven-connors-pullback)
  - [G005 | Moving-Average-Threshold Single-Asset Mean Reversion](#g005--moving-average-threshold-single-asset-mean-reversion)
  - [G006 | Prop-Firm MA-Gap Mean Reversion with Vol-Compression Gate (M5/M15/M30)](#g006--prop-firm-ma-gap-mean-reversion-with-vol-compression-gate-m5m15m30)
  - [G007 | Ten-Day Local-Minimum Pullback Above SMA200](#g007--ten-day-local-minimum-pullback-above-sma200)
  - [G008 | Bollinger Dip in an Uptrend with Multi-Asset Allocation](#g008--bollinger-dip-in-an-uptrend-with-multi-asset-allocation)
  - [G009 | Bollinger(20,2.5)-Cross + RSI(14) Reversal EA (GBP/USD MT4)](#g009--bollinger2025-cross--rsi14-reversal-ea-gbpusd-mt4)
  - [G010 | Crypto Spot Z-Score Mean-Reversion + Rolling-Momentum Signal Blend](#g010--crypto-spot-z-score-mean-reversion--rolling-momentum-signal-blend)
  - [G011 | EUR/USD Z-Score Mean Reversion (OANDA M5)](#g011--eurusd-z-score-mean-reversion-oanda-m5)
  - [G012 | Intraday High-Low Geometric-Midpoint Reversal](#g012--intraday-high-low-geometric-midpoint-reversal)
  - [G013 | N-Bar Contrarian Sign-Flip (mean of last N returns)](#g013--n-bar-contrarian-sign-flip-mean-of-last-n-returns)
  - [G014 | Quote-Based High-Low Multi-Lag Reversion](#g014--quote-based-high-low-multi-lag-reversion)
  - [G015 | Quote-Based Intraday High-Low Reversion](#g015--quote-based-intraday-high-low-reversion)
  - [G016 | RSI Threshold + Higher-High Confirmation Entries (Zerodha Kite, Indian equities)](#g016--rsi-threshold--higher-high-confirmation-entries-zerodha-kite-indian-equities)
  - [G017 | RSI(14) 30/70 Reversal](#g017--rsi14-3070-reversal)
  - [G018 | RSI+MA Mean-Reversion on EUR/USD M1 (OANDA)](#g018--rsima-mean-reversion-on-eurusd-m1-oanda)
  - [G019 | Stochastic-Threshold Mean Reversion on SPY (IB TWS)](#g019--stochastic-threshold-mean-reversion-on-spy-ib-tws)
  - [G020 | Two-Day Dip with Five-Session Hold](#g020--two-day-dip-with-five-session-hold)
  - [G021 | Volume-Adjusted Daily Return Reversal](#g021--volume-adjusted-daily-return-reversal)
  - [G022 | Volume-Adjusted Intraday Reversal and Directional Sizing](#g022--volume-adjusted-intraday-reversal-and-directional-sizing)
- [Pairs, Spread and Statistical Arbitrage](#pairs-spread-and-statistical-arbitrage) (44)
  - [G023 | ADF+Hurst+Half-Life Screened Pair Reversion with Bollinger Distance Entry](#g023--adfhursthalf-life-screened-pair-reversion-with-bollinger-distance-entry)
  - [G024 | Avellaneda-Lee PCA Eigenportfolio S-Score Reversion (Equity_StatArb implementation)](#g024--avellaneda-lee-pca-eigenportfolio-s-score-reversion-equity_statarb-implementation)
  - [G025 | Avellaneda-Lee PCA Eigenportfolio S-Score Reversion](#g025--avellaneda-lee-pca-eigenportfolio-s-score-reversion)
  - [G026 | Bollinger-Band EWA/EWC Pair Spread Reversion](#g026--bollinger-band-ewaewc-pair-spread-reversion)
  - [G027 | Chan Cointegrated-Pair Z-Score Backtester (PairTradeR)](#g027--chan-cointegrated-pair-z-score-backtester-pairtrader)
  - [G028 | Cointegrated Pair Spread Reversion with Risk Limits](#g028--cointegrated-pair-spread-reversion-with-risk-limits)
  - [G029 | Cointegrated Pair Z-Score Mean Reversion](#g029--cointegrated-pair-z-score-mean-reversion)
  - [G030 | Commodity Crack-Spread Cointegration Reversion](#g030--commodity-crack-spread-cointegration-reversion)
  - [G031 | Crude-Oil Calendar-Spread Bollinger Mean Reversion (CL front vs second month)](#g031--crude-oil-calendar-spread-bollinger-mean-reversion-cl-front-vs-second-month)
  - [G032 | Crypto Pair Z-Score Reversion (BTC/ETH, Johansen screening)](#g032--crypto-pair-z-score-reversion-btceth-johansen-screening)
  - [G033 | Decorrelation-Gated Buy-the-Loser Pair Rotation (PEP/KO example)](#g033--decorrelation-gated-buy-the-loser-pair-rotation-pepko-example)
  - [G034 | Kalman Forecast-Error Pair Reversion](#g034--kalman-forecast-error-pair-reversion)
  - [G035 | Kalman-Updated Hedge-Ratio Pair Z-Score Strategy](#g035--kalman-updated-hedge-ratio-pair-z-score-strategy)
  - [G036 | Ornstein–Uhlenbeck Pair-Spread Z-Score](#g036--ornsteinuhlenbeck-pair-spread-z-score)
  - [G037 | Partial-Cointegration Kalman Residual Reversion](#g037--partial-cointegration-kalman-residual-reversion)
  - [G038 | ADF Cointegration Screener over Vanguard ETFs](#g038--adf-cointegration-screener-over-vanguard-etfs)
  - [G039 | Barra Idiosyncratic Residual Return Reversal](#g039--barra-idiosyncratic-residual-return-reversal)
  - [G040 | Bayesian-Optimized OU Half-Life Pair Reversion](#g040--bayesian-optimized-ou-half-life-pair-reversion)
  - [G041 | Commodity Basis Reversal: Outright and Calendar-Spread Variants](#g041--commodity-basis-reversal-outright-and-calendar-spread-variants)
  - [G042 | Commodity Futures Intra-Curve F3/F0 Steepener](#g042--commodity-futures-intra-curve-f3f0-steepener)
  - [G043 | Copula Conditional-Probability Pair Trading](#g043--copula-conditional-probability-pair-trading)
  - [G044 | Daily PCA Idiosyncratic-Residual Reversal](#g044--daily-pca-idiosyncratic-residual-reversal)
  - [G045 | Distance-Selected Pair Spread Reversion](#g045--distance-selected-pair-spread-reversion)
  - [G046 | Dynamic Vasicek Pair Reversion with Market-Factor Adjustment](#g046--dynamic-vasicek-pair-reversion-with-market-factor-adjustment)
  - [G047 | Eight-Session Bollinger Band Pair Reversion](#g047--eight-session-bollinger-band-pair-reversion)
  - [G048 | Granger-Causality-Gated Cointegrated Pair Reversion](#g048--granger-causality-gated-cointegrated-pair-reversion)
  - [G049 | Intraday PCA Residual Mean Reversion](#g049--intraday-pca-residual-mean-reversion)
  - [G050 | Kagi-Filtered Pair Spread Trend/Reversal Switch](#g050--kagi-filtered-pair-spread-trendreversal-switch)
  - [G051 | Kalman Hedge-Ratio Cointegration Pair Reversion](#g051--kalman-hedge-ratio-cointegration-pair-reversion)
  - [G052 | Kalman-Hedge-Ratio Pair Mean Reversion with Beta-Stability Gate](#g052--kalman-hedge-ratio-pair-mean-reversion-with-beta-stability-gate)
  - [G053 | LMAR Hidden-Regime Pair Spread Forecast](#g053--lmar-hidden-regime-pair-spread-forecast)
  - [G054 | Lead–Lag Follower Versus Market Strategy](#g054--leadlag-follower-versus-market-strategy)
  - [G055 | PCA-Factor Residual Stat-Arb Course Strategies (R, NYU Jerzy FRE6871)](#g055--pca-factor-residual-stat-arb-course-strategies-r-nyu-jerzy-fre6871)
  - [G056 | Quasi-Multivariate Correlated-Portfolio Spread Reversal](#g056--quasi-multivariate-correlated-portfolio-spread-reversal)
  - [G057 | Regime- and Momentum-Filtered Adaptive Z-Score Pair Strategy](#g057--regime--and-momentum-filtered-adaptive-z-score-pair-strategy)
  - [G058 | Rolling Cointegration Pair Z-Score Prototype](#g058--rolling-cointegration-pair-z-score-prototype)
  - [G059 | Shumway–Stoffer State-Space Pair Mean Reversion](#g059--shumwaystoffer-state-space-pair-mean-reversion)
  - [G060 | Soybean Crush-Spread Fair-Value Mean Reversion](#g060--soybean-crush-spread-fair-value-mean-reversion)
  - [G061 | Tick-Level Beta-Expected-Price Pair Reversion](#g061--tick-level-beta-expected-price-pair-reversion)
  - [G062 | Tick-Level Kalman Cointegration Error Reversion](#g062--tick-level-kalman-cointegration-error-reversion)
  - [G063 | Treasury Futures Z/T-Score Relative-Value Entry](#g063--treasury-futures-zt-score-relative-value-entry)
  - [G064 | Two-Regime Markov-Switching Pair Spread](#g064--two-regime-markov-switching-pair-spread)
  - [G065 | Wavelet-Filtered Cointegrated Pair Reversion](#g065--wavelet-filtered-cointegrated-pair-reversion)
  - [G066 | Zipline Cointegration Pair Z-Score (z_signal=2, z_exit=0, lookback=200)](#g066--zipline-cointegration-pair-z-score-z_signal2-z_exit0-lookback200)
- [Trend Following and Breakout](#trend-following-and-breakout) (29)
  - [G067 | Mount Lucas (MLM) Futures Trend Index - 200d MA + Universe Vol Gate, Monthly Execution](#g067--mount-lucas-mlm-futures-trend-index---200d-ma--universe-vol-gate-monthly-execution)
  - [G068 | Stochastic-Oversold-Exit + RSI-50 Confirmation with Chandelier(22,2) Trailing Exit](#g068--stochastic-oversold-exit--rsi-50-confirmation-with-chandelier222-trailing-exit)
  - [G069 | 20-Day-High Breakout Trend (Hyperliquid)](#g069--20-day-high-breakout-trend-hyperliquid)
  - [G070 | ATR SuperTrend Direction-Reversal Strategy](#g070--atr-supertrend-direction-reversal-strategy)
  - [G071 | Keltner Mid-Channel Pullback-in-Trend EA](#g071--keltner-mid-channel-pullback-in-trend-ea)
  - [G072 | MBBS Three-Speed EWMA CTA Momentum](#g072--mbbs-three-speed-ewma-cta-momentum)
  - [G073 | Three-Day Pullback Above SMA200](#g073--three-day-pullback-above-sma200)
  - [G074 | Turtle Donchian Reversal (26/10)](#g074--turtle-donchian-reversal-2610)
  - [G075 | Adaptive Renko Trend Following with Partial De-Risking](#g075--adaptive-renko-trend-following-with-partial-de-risking)
  - [G076 | Awesome Oscillator Saucer and Zero-Line Momentum](#g076--awesome-oscillator-saucer-and-zero-line-momentum)
  - [G077 | CTA 1.0 Binary EWM Crossover](#g077--cta-10-binary-ewm-crossover)
  - [G078 | CTA 2.0 Volatility-Scaled EWM Crossover](#g078--cta-20-volatility-scaled-ewm-crossover)
  - [G079 | CTA 3.0 Robust Volatility-Adjusted Oscillator](#g079--cta-30-robust-volatility-adjusted-oscillator)
  - [G080 | CTA 4.0 Robust Oscillator with Risk-Parity Allocation](#g080--cta-40-robust-oscillator-with-risk-parity-allocation)
  - [G081 | CTA 5.0 DCC Correlation-Aware Risk Parity](#g081--cta-50-dcc-correlation-aware-risk-parity)
  - [G082 | Dual Thrust Opening-Range Breakout](#g082--dual-thrust-opening-range-breakout)
  - [G083 | Golden-Cross Moving-Average Trend](#g083--golden-cross-moving-average-trend)
  - [G084 | Leveraged ETF 2 PM Trend-Day Close Trade](#g084--leveraged-etf-2-pm-trend-day-close-trade)
  - [G085 | Local-Extreme Futures Breakout](#g085--local-extreme-futures-breakout)
  - [G086 | MACD-Cross + 10-Bar S/R Break Scalper (XAU/USD M1)](#g086--macd-cross--10-bar-sr-break-scalper-xauusd-m1)
  - [G087 | Median-Renko Bar-Reversal EA](#g087--median-renko-bar-reversal-ea)
  - [G088 | Median-Renko Price/Moving-Average Crossover](#g088--median-renko-pricemoving-average-crossover)
  - [G089 | Median-Renko SuperTrend Reversal EA](#g089--median-renko-supertrend-reversal-ea)
  - [G090 | Median-Renko Two-Moving-Average Crossover](#g090--median-renko-two-moving-average-crossover)
  - [G091 | Parabolic SAR Stop-and-Reverse Trend](#g091--parabolic-sar-stop-and-reverse-trend)
  - [G092 | Rayner-Teo Moving-Average Pullback Strategy (RT_MA)](#g092--rayner-teo-moving-average-pullback-strategy-rt_ma)
  - [G093 | SMA Crossover Trend (50/200; grid-optimized pairs)](#g093--sma-crossover-trend-50200-grid-optimized-pairs)
  - [G094 | Simple-Moving-Average MACD Oscillator Crossover](#g094--simple-moving-average-macd-oscillator-crossover)
  - [G095 | Singular-Spectrum-Analysis Trend Signal](#g095--singular-spectrum-analysis-trend-signal)
- [Momentum, Rotation and Cross-Sectional Factors](#momentum-rotation-and-cross-sectional-factors) (21)
  - [G096 | ANTICOR Anti-Correlation Wealth Transfer](#g096--anticor-anti-correlation-wealth-transfer)
  - [G097 | Baltas 2020 Trend Signal with Realized-Volatility Scaling](#g097--baltas-2020-trend-signal-with-realized-volatility-scaling)
  - [G098 | Moskowitz-Style Time-Series Momentum on Futures](#g098--moskowitz-style-time-series-momentum-on-futures)
  - [G099 | Online Portfolio Selection Algorithm Zoo (OLMAR/PAMR/CWMR/RMR/ANTICOR/CORN/UP/ONS...)](#g099--online-portfolio-selection-algorithm-zoo-olmarpamrcwmrrmranticorcornupons)
  - [G100 | QARMII 252-Day Binary Momentum Portfolio](#g100--qarmii-252-day-binary-momentum-portfolio)
  - [G101 | Quadratic Dip-Depth Sizing Formula on VTI/QQQ (SGOV cash park)](#g101--quadratic-dip-depth-sizing-formula-on-vtiqqq-sgov-cash-park)
  - [G102 | Top-50 S&P500 12-Month Momentum Portfolio (with HQM low-quality-momentum filter variant)](#g102--top-50-sp500-12-month-momentum-portfolio-with-hqm-low-quality-momentum-filter-variant)
  - [G103 | Top-One Positive-Slope Momentum Rotation](#g103--top-one-positive-slope-momentum-rotation)
  - [G104 | 232-Session-Lagged Industry-Relative Momentum Factor](#g104--232-session-lagged-industry-relative-momentum-factor)
  - [G105 | Beta-Divided Multi-Period Return Reversal Factor](#g105--beta-divided-multi-period-return-reversal-factor)
  - [G106 | Cross-Sectional 252-Session Commodity Momentum](#g106--cross-sectional-252-session-commodity-momentum)
  - [G107 | Cross-Sectional Commodity Basis Momentum](#g107--cross-sectional-commodity-basis-momentum)
  - [G108 | Cross-Sectional Futures Carry](#g108--cross-sectional-futures-carry)
  - [G109 | Cross-Sectional Skewness Risk-Premium Signal](#g109--cross-sectional-skewness-risk-premium-signal)
  - [G110 | Daily Plus Intraday Beta-Adjusted Return Forecast](#g110--daily-plus-intraday-beta-adjusted-return-forecast)
  - [G111 | Intraday Beta-Divided Return Reversal](#g111--intraday-beta-divided-return-reversal)
  - [G112 | Intraday Market-Weighted Beta-Residual Returns](#g112--intraday-market-weighted-beta-residual-returns)
  - [G113 | Market-Weighted Beta-Residual Return Factor](#g113--market-weighted-beta-residual-return-factor)
  - [G114 | NSE Stock Scanner - Momentum/Swing Screening Signals](#g114--nse-stock-scanner---momentumswing-screening-signals)
  - [G115 | Robust-Value Percentile-Composite Portfolio (PE/PB/PS/EV-EBITDA/EV-GP)](#g115--robust-value-percentile-composite-portfolio-pepbpsev-ebitdaev-gp)
  - [G116 | Volatility-Ratio-Weighted Return Reversal Factor](#g116--volatility-ratio-weighted-return-reversal-factor)
- [Formulaic Alpha Factor Libraries](#formulaic-alpha-factor-libraries) (7)
  - [G117 | QLib Alpha158/Alpha360 Handcrafted Factor Libraries](#g117--qlib-alpha158alpha360-handcrafted-factor-libraries)
  - [G118 | Alpha101 Formulaic Alpha Library](#g118--alpha101-formulaic-alpha-library)
  - [G119 | Alpha101-009 Five-Day Conditional Reversal](#g119--alpha101-009-five-day-conditional-reversal)
  - [G120 | Alpha101-012 Volume-Confirmed One-Day Reversal](#g120--alpha101-012-volume-confirmed-one-day-reversal)
  - [G121 | Alpha101-101 Intraday Candle-Location Factor](#g121--alpha101-101-intraday-candle-location-factor)
  - [G122 | Alpha191 Formulaic Alpha Library](#g122--alpha191-formulaic-alpha-library)
  - [G123 | Alpha191-020 Six-Session Price Momentum](#g123--alpha191-020-six-session-price-momentum)
- [Price Action, Candlestick and Smart-Money Patterns](#price-action-candlestick-and-smart-money-patterns) (7)
  - [G124 | Bollinger-Band Touch then Pullback-to-MA Continuation Entry](#g124--bollinger-band-touch-then-pullback-to-ma-continuation-entry)
  - [G125 | ICT Smart-Money-Concepts Signal Primitives (FVG, Order Blocks, BOS/CHoCH, Liquidity, Premium/Discount)](#g125--ict-smart-money-concepts-signal-primitives-fvg-order-blocks-boschoch-liquidity-premiumdiscount)
  - [G126 | STAR-EA ICT 12-Scenario Engine with AMD Phase Gating](#g126--star-ea-ict-12-scenario-engine-with-amd-phase-gating)
  - [G127 | Bollinger Bands Bottom-W Pattern Reversal](#g127--bollinger-bands-bottom-w-pattern-reversal)
  - [G128 | Heikin-Ashi Consecutive-Candle Trend Reversal](#g128--heikin-ashi-consecutive-candle-trend-reversal)
  - [G129 | RSI Head-and-Shoulders Pattern Reversal](#g129--rsi-head-and-shoulders-pattern-reversal)
  - [G130 | Shooting-Star Candlestick Reversal](#g130--shooting-star-candlestick-reversal)
- [Volatility and Options-Derived Signals](#volatility-and-options-derived-signals) (9)
  - [G131 | Long/Short Straddle Volatility Trading (mcf-long-short IBKR)](#g131--longshort-straddle-volatility-trading-mcf-long-short-ibkr)
  - [G132 | Options Wheel (CSP -> Covered Call) - alpacahq template](#g132--options-wheel-csp---covered-call---alpacahq-template)
  - [G133 | Short-Term-Reversal Option Setups with Staged Entry (randomwalkhan)](#g133--short-term-reversal-option-setups-with-staged-entry-randomwalkhan)
  - [G134 | Systematic Equity-Options Volatility Research (anthonymakarewicz)](#g134--systematic-equity-options-volatility-research-anthonymakarewicz)
  - [G135 | AI-Trader NSE Options - Dual ML + Institutional Flow](#g135--ai-trader-nse-options---dual-ml--institutional-flow)
  - [G136 | Gamma Scalping Delta-Hedge Template (alpacahq)](#g136--gamma-scalping-delta-hedge-template-alpacahq)
  - [G137 | Heikin-Ashi-Style Option Long Straddle Entry](#g137--heikin-ashi-style-option-long-straddle-entry)
  - [G138 | Options Strategy Backtest Cookbook Set (PyPatel + Option Greeks book)](#g138--options-strategy-backtest-cookbook-set-pypatel--option-greeks-book)
  - [G139 | VIX Term-Structure and Pair-Spread Volatility Study](#g139--vix-term-structure-and-pair-spread-volatility-study)
- [Calendar, Session and Overnight Effects](#calendar-session-and-overnight-effects) (7)
  - [G140 | Equity Overnight Return Hold](#g140--equity-overnight-return-hold)
  - [G141 | Close-to-Open Gap Reversal Forecast](#g141--close-to-open-gap-reversal-forecast)
  - [G142 | Gao 2018 Equity ETF Intraday Momentum](#g142--gao-2018-equity-etf-intraday-momentum)
  - [G143 | Hour-of-Day Session Gating (HourBasedStrategy)](#g143--hour-of-day-session-gating-hourbasedstrategy)
  - [G144 | London Session Opening-Range Breakout](#g144--london-session-opening-range-breakout)
  - [G145 | Month-End Long After Negative Prior Month](#g145--month-end-long-after-negative-prior-month)
  - [G146 | Weekday-Conditioned Beta-Adjusted Return Forecast](#g146--weekday-conditioned-beta-adjusted-return-forecast)
- [News, Sentiment and Alternative Data](#news-sentiment-and-alternative-data) (12)
  - [G147 | Binance News-Sentiment Bot - Headline Sentiment -> Buy New Listings/Alts](#g147--binance-news-sentiment-bot---headline-sentiment---buy-new-listingsalts)
  - [G148 | Earnings-Surprise Drift Factor](#g148--earnings-surprise-drift-factor)
  - [G149 | WSB Mention-Frequency Deviation Long/Short](#g149--wsb-mention-frequency-deviation-longshort)
  - [G150 | 10-K Filing NLP Long-Term Growth Prediction](#g150--10-k-filing-nlp-long-term-growth-prediction)
  - [G151 | Analyst Price-Target Deviation Reversion](#g151--analyst-price-target-deviation-reversion)
  - [G152 | Beta-Adjusted Analyst Rating and Price-Move Interaction](#g152--beta-adjusted-analyst-rating-and-price-move-interaction)
  - [G153 | Coverage-Filtered Asymmetric Analyst Rating Revisions](#g153--coverage-filtered-asymmetric-analyst-rating-revisions)
  - [G154 | EWMA Analyst-Rating Change Mean Reversion](#g154--ewma-analyst-rating-change-mean-reversion)
  - [G155 | Hard-to-Borrow Fee Crowding Signal](#g155--hard-to-borrow-fee-crowding-signal)
  - [G156 | Scheduled-News Straddle Spike (EarnForex News Trader EA)](#g156--scheduled-news-straddle-spike-earnforex-news-trader-ea)
  - [G157 | Sell-Side Analyst Estimate Revision Forecast](#g157--sell-side-analyst-estimate-revision-forecast)
  - [G158 | Twitter Sentiment Stock-Movement Prediction](#g158--twitter-sentiment-stock-movement-prediction)
- [Supervised Machine Learning Prediction](#supervised-machine-learning-prediction) (15)
  - [G159 | Machine-Learning-for-Trading (Stefan Jansen) - Book Strategy Compendium](#g159--machine-learning-for-trading-stefan-jansen---book-strategy-compendium)
  - [G160 | QLib Model Zoo on Daily Alpha Factors (LightGBM/MLP/LSTM/ALSTM/TFT/Transformer)](#g160--qlib-model-zoo-on-daily-alpha-factors-lightgbmmlplstmalstmtfttransformer)
  - [G161 | CUSUM-Filtered Event + Triple-Barrier-Label ML Long/Short Pipeline](#g161--cusum-filtered-event--triple-barrier-label-ml-longshort-pipeline)
  - [G162 | Dahshan228 FTMO Bot - LightGBM Equity Signal + BTC/FX Trend (2015-2025 validated)](#g162--dahshan228-ftmo-bot---lightgbm-equity-signal--btcfx-trend-2015-2025-validated)
  - [G163 | Pattern-Similarity k-NN Forecast (normalized %change shape matching)](#g163--pattern-similarity-k-nn-forecast-normalized-change-shape-matching)
  - [G164 | Surpriver - Anomaly-Detection Pre-Move Screener](#g164--surpriver---anomaly-detection-pre-move-screener)
  - [G165 | ai_quant_trade - One-Stop AI Quant Platform Strategy + Factor-Mining + LLM Set](#g165--ai_quant_trade---one-stop-ai-quant-platform-strategy--factor-mining--llm-set)
  - [G166 | robertmartin8 MachineLearningStocks - Fundamental+Technical Sklearn Prediction](#g166--robertmartin8-machinelearningstocks---fundamentaltechnical-sklearn-prediction)
  - [G167 | Classical ML Stock-Predictor Set (SVM/KNN/GP/AdaBoost/QDA/ARIMA/LSTM course repos)](#g167--classical-ml-stock-predictor-set-svmknngpadaboostqdaarimalstm-course-repos)
  - [G168 | Decision-Tree Leg-Selection for Pair Rotation](#g168--decision-tree-leg-selection-for-pair-rotation)
  - [G169 | Dollar-Bar + Triple-Barrier ML Strategy (QuantConnect, MOEX tick)](#g169--dollar-bar--triple-barrier-ml-strategy-quantconnect-moex-tick)
  - [G170 | Hurst-Exponent Stationarity Screen + ARIMA Forecast Filter (Indian equities)](#g170--hurst-exponent-stationarity-screen--arima-forecast-filter-indian-equities)
  - [G171 | PCA-Reduced SVR Pair Fair-Value Voting](#g171--pca-reduced-svr-pair-fair-value-voting)
  - [G172 | SVM 90-Day Multi-Feature Direction Classifier](#g172--svm-90-day-multi-feature-direction-classifier)
  - [G173 | intelligent-trading-bot - Feature-Engineered Signal Generation (asavinov)](#g173--intelligent-trading-bot---feature-engineered-signal-generation-asavinov)
- [Deep Learning Sequence Models](#deep-learning-sequence-models) (4)
  - [G174 | Encoder-Decoder LSTM Multi-Step Pair-Spread Forecast](#g174--encoder-decoder-lstm-multi-step-pair-spread-forecast)
  - [G175 | LSTM/NN Return-Prediction Alpha with Portfolio Sim (MS&E 448)](#g175--lstmnn-return-prediction-alpha-with-portfolio-sim-mse-448)
  - [G176 | RNN Forecasted Pair-Spread Momentum](#g176--rnn-forecasted-pair-spread-momentum)
  - [G177 | StockFormer - Hybrid Predictive-Coding Trading Machine](#g177--stockformer---hybrid-predictive-coding-trading-machine)
- [Reinforcement Learning Agents](#reinforcement-learning-agents) (5)
  - [G178 | FinRL Ensemble DRL Agent (PPO/A2C/DDPG/TD3/SAC) on Dow30 Daily](#g178--finrl-ensemble-drl-agent-ppoa2cddpgtd3sac-on-dow30-daily)
  - [G179 | FinRL-DeepSeek - LLM-Infused Risk-Sensitive RL](#g179--finrl-deepseek---llm-infused-risk-sensitive-rl)
  - [G180 | Deep RL Trading Agent Zoo (grouped: DQN/A2C/PPO/DDQN/A3C across crypto+equity)](#g180--deep-rl-trading-agent-zoo-grouped-dqna2cppoddqna3c-across-cryptoequity)
  - [G181 | Deep Reinforcement Learning Pair-Position Policy](#g181--deep-reinforcement-learning-pair-position-policy)
  - [G182 | LSTM Policy-Gradient Pair-Trading Agent](#g182--lstm-policy-gradient-pair-trading-agent)
- [Crypto Multi-Indicator Bot Strategies](#crypto-multi-indicator-bot-strategies) (14)
  - [G183 | BBGO Strategy Suite - Supertrend, TrendTrader, PivotShort, FlashCrash, EWO, FactorZoo, Drift](#g183--bbgo-strategy-suite---supertrend-trendtrader-pivotshort-flashcrash-ewo-factorzoo-drift)
  - [G184 | Freqtrade Community Strategy Collection - Indicator Combo Families](#g184--freqtrade-community-strategy-collection---indicator-combo-families)
  - [G185 | Haehnchen Dip-Catcher / DCA-Dipper / OBV Pump-Dump / Pivot-Reversal Set](#g185--haehnchen-dip-catcher--dca-dipper--obv-pump-dump--pivot-reversal-set)
  - [G186 | KAC Index v1 - TOTAL3-Indexed ATR+CCI Altcoin Relative-Strength Entries](#g186--kac-index-v1---total3-indexed-atrcci-altcoin-relative-strength-entries)
  - [G187 | KAC Index v2 - TOTAL3-Indexed ATR>2.75 Single-Filter Entries](#g187--kac-index-v2---total3-indexed-atr275-single-filter-entries)
  - [G188 | NostalgiaForInfinity (NFI) - Protected Dip-Buying Grid with DCA](#g188--nostalgiaforinfinity-nfi---protected-dip-buying-grid-with-dca)
  - [G189 | Simple VWAP v1 - Constant-Exposure VWAP-Band Mean Reversion with DCA](#g189--simple-vwap-v1---constant-exposure-vwap-band-mean-reversion-with-dca)
  - [G190 | Binance Overbought-Composite Short (RSI70 + BB mid + MACD + EMA20 filter)](#g190--binance-overbought-composite-short-rsi70--bb-mid--macd--ema20-filter)
  - [G191 | Crypto-Signal Alert Strategy Set (RSI/MACD/Ichimoku/OBV/Momentum/Breakout)](#g191--crypto-signal-alert-strategy-set-rsimacdichimokuobvmomentumbreakout)
  - [G192 | Gekko Classic Indicator Strategy Set (RSI/MACD/DEMA/PPO/StochRSI/TSI/UO/CCI/ADX)](#g192--gekko-classic-indicator-strategy-set-rsimacddemappostochrsitsiuocciadx)
  - [G193 | Gekko-Strategies Community Set with Backtest Reports](#g193--gekko-strategies-community-set-with-backtest-reports)
  - [G194 | Jesse Framework Strategy Stubs (test/regression strategies)](#g194--jesse-framework-strategy-stubs-testregression-strategies)
  - [G195 | Ninjabot Example Strategies (Go framework examples)](#g195--ninjabot-example-strategies-go-framework-examples)
  - [G196 | PyCryptoBot EMA12/26 Golden-Cross + MACD Confirmation Bot](#g196--pycryptobot-ema1226-golden-cross--macd-confirmation-bot)
- [Prediction-Market Strategies](#prediction-market-strategies) (1)
  - [G197 | Prediction-Market Strategy Families (convergence, momentum, copy-trade, last-seconds, dump-hedge)](#g197--prediction-market-strategy-families-convergence-momentum-copy-trade-last-seconds-dump-hedge)
- [Execution-Aware and Miscellaneous](#execution-aware-and-miscellaneous) (12)
  - [G198 | Direction-Agnostic Wide-Stop Ratchet-Exit Framework (mr-scrooge v6)](#g198--direction-agnostic-wide-stop-ratchet-exit-framework-mr-scrooge-v6)
  - [G199 | Bid-Ask Depth Imbalance Forecast](#g199--bid-ask-depth-imbalance-forecast)
  - [G200 | Daily/Intraday Beta-Adjusted Order-Flow Imbalance](#g200--dailyintraday-beta-adjusted-order-flow-imbalance)
  - [G201 | Differenced Intraday Order-Flow Momentum](#g201--differenced-intraday-order-flow-momentum)
  - [G202 | Indicator Ternary-Signal Portfolio Engine (MACD zero-cross, Bollinger bands, CMF)](#g202--indicator-ternary-signal-portfolio-engine-macd-zero-cross-bollinger-bands-cmf)
  - [G203 | MA-Slope-Biased Martingale](#g203--ma-slope-biased-martingale)
  - [G204 | Momentum-Following + Implementation-Shortfall Execution Study (Algo-Trading-14)](#g204--momentum-following--implementation-shortfall-execution-study-algo-trading-14)
  - [G205 | Multi-Pair Martingale Bot (Binance)](#g205--multi-pair-martingale-bot-binance)
  - [G206 | OpenAlgo Execution-Skills - VectorBT+Live Multi-Strategy Set](#g206--openalgo-execution-skills---vectorbtlive-multi-strategy-set)
  - [G207 | Optimal Execution Algorithms Set (DP / Frank-Wolfe / Market-Impact)](#g207--optimal-execution-algorithms-set-dp--frank-wolfe--market-impact)
  - [G208 | Order-Book Insideness-Return Interaction](#g208--order-book-insideness-return-interaction)
  - [G209 | ai-trader - Backtrader 20+ Strategy Collection](#g209--ai-trader---backtrader-20-strategy-collection)

---

## Single-Asset Mean Reversion (oscillators, bands, z-score)

### G001 | Daily-Checkpoint Oversold-Bounce with Trend-Midpoint Exits (Mag10 + index ETFs)
- priority: High — Fully specified daily-bar oversold system with complete exits - immediately testable on the declared universe.
- direction: long | market: US large-caps (AAPL MSFT GOOGL AMZN META NVDA TSLA AVGO AMD ORCL) + SPY/IWM/DIA | timeframe: 1d (daily-native)
- indicators: RSI(14), 20-day SMA, 50-day high, 20-day average volume
- Overview: Oversold-bounce on a fixed 13-name liquid universe, checked daily: buy on RSI(14)<30 below the 20-MA or a >=4% capitulation day on 1.5x volume, prefer the name furthest below its 50-day high; exit at the 20-MA touch, -5% stop, or 10-day time stop. Clean, complete, daily-bar retail mean-reversion spec with honest warnings about retail overtrading.
- Stock or ETF Criteria: Fixed universe: 10 mega-caps + SPY/IWM/DIA only.
- Stocks to avoid and known failure conditions: Author's own caveat: retail active trading underperforms; RSI(14)<30 is a weak oversold trigger vs RSI(2) family; quarterly kill-switch review.
- Entry conditions: RSI(14)<30 AND close<20-day SMA; OR close >=4% below prior close with volume >1.5x 20d avg. Tiebreak: largest drop from 50-day high.
- Exit conditions: First of: close >=20-day SMA (TP), close >=5% below entry (SL), or 10 sessions held.
- Trade management: $100 per trade at starter size; one position per signal; daily checkpoint.
- Target: Mean reversion to the 20-day SMA.
- Risk factors and risk management: 5% stop, 10-day time stop, fixed universe, quarterly strategy kill-switch.
- Best known candlestick timeframe(s): Daily bars (native).
- reported_metrics: none reported
- notes: RSI14<30 entry is the weak link - try RSI(2)<5 variant at implementation.
- sources: [FarzamHejaziK/claude-investment-assistant — complete rule spec](https://github.com/FarzamHejaziK/claude-investment-assistant/blob/9607f6b52214f20e340c1c27491766b78b7acc78/strategies/active-trading.example.md)

### G002 | Bollinger(30,2) Band-Touch Mean Reversion with Neutral Zone (EUR/USD)
- priority: Medium — The neutral-zone cost-efficiency insight is real and directly transferable to daily-bar S011 variants.
- direction: long/short | market: FX EUR/USD (6h bars) | timeframe: 6h (intraday-adaptable)
- indicators: SMA30 +/- 2 std Bollinger bands
- Overview: Band-touch mean reversion that stays FLAT between the bands: long only below the lower band, short only above the upper band, neutral otherwise. The neutral zone is the point of the strategy - fewer, higher-conviction trades and materially lower transaction costs than always-in-market signals; the notebooks show it outperforming buy-and-hold with less risk after costs.
- Stock or ETF Criteria: Demonstrated on EUR/USD 6h; any liquid instrument.
- Stocks to avoid and known failure conditions: Band-riding trends (breakout days) produce consecutive losing fades; flat zones reduce trade count - few signals in quiet markets.
- Entry conditions: Long when close < lower band (SMA30 - 2std); short when close > upper band; flat between bands.
- Exit conditions: Re-enter neutral zone => flat (position = sign relative to bands).
- Trade management: +1/-1/0 unit positions; proportional transaction cost modeled.
- Target: Reversion inside the band envelope.
- Risk factors and risk management: No stop beyond the flat-on-reentry rule; sustained trend = continuous adverse position.
- Best known candlestick timeframe(s): 6h bars in code; daily adaptation = classic Bollinger-band MR (cf. S011).
- reported_metrics: Notebook claims outperformance vs buy-and-hold with lower risk after costs on EUR/USD sample (repo-claimed, unverified).
- external_refs: Udemy 'Algorithmic Trading A-Z' course lineage
- notes: Same rule family as S011 - crossref if catalog version suffices; kept full for the flat-zone framing.
- sources: [BearsOnMars/Algorithmic_Trading — SMA=30 dev=2; neutral-zone logic and cost analysis](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Mean-Reversion/Mean-Reversion%20%28Bollinger%20Bands%29%20Strategy%20-%20Design%20and%20Backtest.ipynb); [BearsOnMars/Algorithmic_Trading — train/forward split](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Mean-Reversion/Mean-Reversion%20%28Bollinger%20Bands%29%20Backtester%20class%20with%20OOP.ipynb)

### G003 | Daily High-Low Geometric-Midpoint Reversal
- priority: Medium — The signal and short holding horizon are explicit, but no stand-alone cost-aware OOS results are claimed.
- direction: long/short | market: US equities | timeframe: 1d (daily-native)
- indicators: Close divided by geometric mean of daily high and low; winsorized, industry demeaned, lagged signals
- Overview: The signal treats a close far from the day's geometric high-low midpoint as a temporary dislocation and fits lagged values to forward returns. Industry demeaning seeks stock-specific rather than sector-wide reversals.
- Stock or ETF Criteria: US equities with daily OHLC data and Barra industry classifications; source describes an expandable liquid universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute hl0=close/sqrt(high*low), winsorize by date, demean within ind1, then use lagged hl1..hlN; regress against the configured forward-return horizon and apply the fitted coefficients.
- Exit conditions: Recompute the forecast in the next daily simulation; no discrete stop or price target is specified.
- Trade management: Daily alpha forecasts feed the bsim portfolio optimizer; kappa/notional constraints are configured by the simulation engine.
- Target: Expected 1–3 day forecast horizon in the module description.
- Risk factors and risk management: The signal has high turnover; the strategy relies on proprietary market/industry datasets and external optimizer configuration.
- Best known candlestick timeframe(s): Daily bars are native and the code describes a 1–3 session horizon.
- reported_metrics: none reported
- notes: The module generates forecasts rather than order instructions; execution and allocation are handled by separate simulation engines.
- sources: [noterminusgit/statarb — Daily high-low signal construction, lagging, regression fitting and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/hl.py#L141-L423)

### G004 | Double-Seven Connors Pullback
- priority: Medium — 
- overlap: S006 (existing catalog) — Calls the catalog's Double7 family; exact defaults are inside private `vivace` and the notebook applies an adjusted-close daily backtest.
- direction:  | market: US equities | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [rolling-panda-san/notebooks — Double7Connors signal; code cell 16](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/equity_short_term_trading_connors.ipynb)

### G005 | Moving-Average-Threshold Single-Asset Mean Reversion
- priority: Medium — Entry and exit are exact and costs are supported, but the fixed dollar threshold is not portable across prices.
- direction: long/short | market: US-listed gold/energy equities; examples GDX and GLD | timeframe: 1d (daily-native)
- indicators: Rolling SMA and price distance from the SMA
- Overview: The backtester trades deviations from a simple moving average, entering against large price dislocations and exiting when price crosses back through the average. It is a transparent threshold mean-reversion rule.
- Stock or ETF Criteria: Liquid ETFs/stocks; notebook examples use GDX and GLD.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: With SMA=25 and threshold=$5, short when close−SMA>$5 and long when close−SMA<−$5; hold the last position between entries until the distance changes sign.
- Exit conditions: Exit when the price-SMA distance crosses zero; position is shifted one bar before returns are applied.
- Trade management: Vectorized single-asset positions; transaction cost is deducted on each position change (example uses 0.1%).
- Target: Mean reversion to the rolling SMA.
- Risk factors and risk management: Absolute-dollar thresholds do not scale across instruments or price regimes; example threshold is tuned on the tested sample.
- Best known candlestick timeframe(s): Daily bars are native; no intraday test is shown.
- reported_metrics: none reported
- notes: The book-derived implementation applies one-bar signal delay before strategy returns.
- sources: [edgetrader/mean-reversion-strategy — SMA-distance entries, zero-cross exit, and transaction-cost application](https://github.com/edgetrader/mean-reversion-strategy/blob/194240de64bf4e99592e762b2677b53dbb65cf3c/src/MRVectorBacktester.py#L40-L68); [edgetrader/mean-reversion-strategy — Example GDX/GLD backtests with SMA/threshold values](https://github.com/edgetrader/mean-reversion-strategy/blob/194240de64bf4e99592e762b2677b53dbb65cf3c/src/MRVectorBacktester.py#L75-L84)

### G006 | Prop-Firm MA-Gap Mean Reversion with Vol-Compression Gate (M5/M15/M30)
- priority: Medium — Interesting structural idea (deviation entry gated on vol compression) + genuinely complete risk framework; intraday-native.
- direction: long/short | market: 12 FX majors + commodities (MT5) | timeframe: M5/M15/M30 (intraday-adaptable)
- indicators: EMA360 fast MA, EMA20, RSI(20), fast/slow ATR, min_ma_gap % deviation, volume-weighted spread
- Overview: Fades large deviations from a slow EMA, but ONLY in compressed volatility: buy requires close < EMA360 - min_ma_gap% AND fast ATR < slow ATR AND RSI(20) still declining below the upper bound (entering early in the deviation, not after reversal). Sell mirror. Wrapped in serious prop-firm risk machinery: fixed $ risk, 1:5 RR, daily loss cap 5%, max DD 10%, multi-stage ATR trailing + profit-lock, order expiration, session rules.
- Stock or ETF Criteria: 12 majors + XAUUSD/USOIL/XAGUSD (commodity-specific trailing per symbol); spread filter required.
- Stocks to avoid and known failure conditions: Deviation trades keep fading into trends - the ATR-compression gate helps but whipsaws remain; commodity trailing differs per symbol (curve-fit surface).
- Entry conditions: Buy: close < EMA360 - gap AND atr_fast < atr_slow AND rsi[t-1] > rsi[t] AND rsi < rsi_upper(70). Sell mirror with rsi > rsi_lower(30). Up to MAX_TRADES_PER_SYMBOL_TF concurrent.
- Exit conditions: Multi-stage trailing: profit_units-based ratchet + ATR-factor trailing; TP derived as entry +/- SL_distance * RISK_REWARD_RATIO(5); order expiration.
- Trade management: $50 fixed risk per trade, position size = risk/SL; daily loss limit 5% equity; max DD 10% kill; session window Sun-Fri Lagos GMT+1.
- Target: 1:5 risk-reward take-profit.
- Risk factors and risk management: Fixed-dollar risk sizing, equity daily-loss and max-DD circuit breakers, spread monitoring.
- Best known candlestick timeframe(s): M5-M30 native; a daily adaptation (deviation from EMA200 in vol-compression) is testable but unproven.
- reported_metrics: none reported
- notes: Entry buys into a STILL-FALLING deviation (rsi declining) - early-entry mean reversion, unusual.
- sources: [Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot — buy_condition/sell_condition gates + trailing/TP logic](https://github.com/Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot/blob/28d4d4a828716ad16e38560a45d63ebf5fef164e/engine.py); [Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot — MIN_MA_GAP, ATR1/ATR2, RSI_PERIOD, risk constants](https://github.com/Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot/blob/28d4d4a828716ad16e38560a45d63ebf5fef164e/config.py); [Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot — daily loss / max DD limits](https://github.com/Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot/blob/28d4d4a828716ad16e38560a45d63ebf5fef164e/risk_management.py)

### G007 | Ten-Day Local-Minimum Pullback Above SMA200
- priority: Medium — 
- overlap: S006 (existing catalog) — Uses a 10-session local minimum instead of the catalog's 7-session low, with a 200-day trend gate and five-session signal carry.
- direction:  | market: US equities | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [rolling-panda-san/notebooks — 10-session local minimum, 200-session trend gate; code cell 10](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/equity_short_term_trading_connors.ipynb)

### G008 | Bollinger Dip in an Uptrend with Multi-Asset Allocation
- priority: Low — A regime-filtered dip-buying thesis is plausible but the exit and some indicator defaults are unspecified.
- direction: long | market: US equities; Backtrader sample | timeframe: 1d (daily-native)
- indicators: Bollinger Bands and market uptrend filter
- Overview: The intermediate strategy buys a lower-band pullback only when the broad market is considered uptrending, combining single-name mean reversion with a market-regime gate.
- Stock or ETF Criteria: Multiple equity data feeds with Bollinger indicators.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each asset, enter long on a lower Bollinger Band touch while `market_uptrending()` is true; the source's exact band/market filter parameters are controlled by the Backtrader strategy params.
- Exit conditions: The code does not specify a fixed profit target or maximum holding period; exit mechanics depend on the strategy's ongoing band/portfolio state.
- Trade management: Allocates the configured fixed size to eligible assets; no costs or stop are specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily data are used by the example.
- reported_metrics: none reported
- notes: Repository's backtest class is demonstrative and does not report a robust out-of-sample test.
- sources: [yungalyx/NoisyBoyAlgotrader — Market-uptrend filter and per-asset Bollinger dip entry](https://github.com/yungalyx/NoisyBoyAlgotrader/blob/6c5783b0a8d602c1455e34e9d9eb664edc7b0b43/src/strategies/intermediate.py#L6-L44)

### G009 | Bollinger(20,2.5)-Cross + RSI(14) Reversal EA (GBP/USD MT4)
- priority: Low — Band-cross + RSI confluence is a known weak-to-moderate signal; unverifiable return claim.
- direction: long/short | market: FX GBP/USD (MT4) | timeframe: chart TF (unspecified) (intraday-adaptable)
- indicators: Bollinger(20, 2.5sd) bands on prior bars, RSI(14) 30/70
- Overview: MQL4 EA that buys the bar AFTER a close below the lower Bollinger(20,2.5) band: previous bar bearish AND closed below the band AND RSI<30, with the band cross having just occurred (prior-close above prev-band). Sell mirror at the upper band with RSI>70. Includes trailing logic and a prop-challenge profit-target check. Classic band-cross + RSI confluence fade.
- Stock or ETF Criteria: GBP/USD; any MT4 symbol.
- Stocks to avoid and known failure conditions: Prop-firm framing suggests curve-fitted risk gates; README's 76%/yr claim is marketing-grade - assume overfit.
- Entry conditions: Buy if Close[1]<lowerBB(20,2.5) AND Close[1]<Open[1] AND RSI(14)<30 AND Close[2]>prev_lowerBB (fresh cross). Sell mirror (Close[1]>upperBB, bullish bar, RSI>70, fresh cross).
- Exit conditions: SL/TP per order (sl/Tp vars) + trailing buy/sell logic.
- Trade management: Fixed lotsize; challenge-profit-target gate on new entries.
- Target: Fixed TP per trade.
- Risk factors and risk management: Per-trade SL/TP + trailing.
- Best known candlestick timeframe(s): Chart TF unspecified in code (works off bar arrays); daily adaptation = band-cross + RSI(2/14) fade, cf. S011.
- reported_metrics: README claims >76%/yr average since 2011 (repo-claimed, unverified - treat as suspect).
- notes: Overlap S011 family; fresh-cross requirement is the distinguishing detail.
- sources: [iamshakibulislam/gbp-usd-forex-trading-mean-reversion-bot — band-cross entry conditions, trailing](https://github.com/iamshakibulislam/gbp-usd-forex-trading-mean-reversion-bot/blob/47621d188d8bfb7e690afef670f932b2f94ad45a/pattern_trader.mq4)

### G010 | Crypto Spot Z-Score Mean-Reversion + Rolling-Momentum Signal Blend
- priority: Low — Naive signal blend on 2 assets; weak expected edge, but daily-native and testable.
- direction: long | market: crypto spot (BTC, ETH via CoinGecko) | timeframe: 1d (daily-native)
- indicators: Rolling z-score of price; rolling sum of pct returns (momentum); sign-vote signal combiner
- Overview: Combines a rolling z-score mean-reversion signal (-zscore) and a rolling momentum signal on BTC/ETH daily prices, then takes the sign of the average as position. Blends two offsetting signals - mean reversion dominates unless momentum is extreme. Educational but directly testable.
- Stock or ETF Criteria: not specified in source. Suggested: liquid large-cap crypto (BTC, ETH).
- Stocks to avoid and known failure conditions: not specified in source. Suggested: avoid illiquid alts and stable pairs; blended signal neutralizes conflicting regimes - expect few clean trades in choppy tape.
- Entry conditions: Position = sign( mean( -zscore(close, window), rolling_sum(pct_change, window) ) ). Window not specified in source. Suggested: window=20.
- Exit conditions: Position flips when combined signal crosses zero; effectively always-in-market long/flat.
- Trade management: not specified in source. Suggested: 1 unit per asset; transaction_cost deducted per position change in backtest_portfolio.
- Target: not specified in source. Suggested: hold until signal sign change.
- Risk factors and risk management: Backtest deducts transaction_cost * |position diff|; separate Sharpe-max long-only portfolio optimizer over the two assets. Suggested: cap per-asset weight 100%.
- Best known candlestick timeframe(s): Daily bars (returns pct_change daily).
- reported_metrics: none reported
- notes: optimize_portfolio is Sharpe-max long-only over the two cryptos, separate from the signal blend.
- sources: [notaconduit/Statistical-Arbitrage-in-Cryptocurrencies — mean_reversion/momentum/combine_signals/backtest_portfolio/optimize_portfolio](https://github.com/notaconduit/Statistical-Arbitrage-in-Cryptocurrencies/blob/e1cb407f5829c71a423d238f2ed70d9596f3e046/trading.py)

### G011 | EUR/USD Z-Score Mean Reversion (OANDA M5)
- priority: Low — Intraday-only trivial z-score fade; daily analog already in catalog.
- direction: long/short | market: FX EUR/USD via OANDA | timeframe: M5 (intraday-only)
- indicators: Rolling 20-bar SMA and std of closes -> z-score
- Overview: Minimal z-score mean-reversion loop on EUR/USD 5-minute candles: sell when z-score exceeds the overbought threshold, buy when below oversold. Pure price-stretch fade.
- Stock or ETF Criteria: EUR/USD (config currency_pairs).
- Stocks to avoid and known failure conditions: No trend filter - bleeds in one-way FX moves; M5 z-score edges are spread-sensitive.
- Entry conditions: z = (close - SMA20)/std20 on M5; sell at z > overbought_threshold, buy at z < oversold_threshold (config defaults in config.ini).
- Exit conditions: Fixed pip stop_loss and take_profit per config; no signal-based exit.
- Trade management: Fixed units per signal; one position per symbol.
- Target: Fixed take_profit pips.
- Risk factors and risk management: Fixed pip stop; no sizing logic.
- Best known candlestick timeframe(s): M5 native; a daily z-score analog would just be S011 (Bollinger/z-score MR).
- reported_metrics: none reported
- notes: Signal thresholds externalized in config.ini; overbought/oversold values are user-tuned.
- sources: [XBT3K/MeanReversionAlgo — MeanReversion.check_signal z-score logic](https://github.com/XBT3K/MeanReversionAlgo/blob/7ac620cf3c030cf0a6401773e0dc8827d7172449/main.py)

### G012 | Intraday High-Low Geometric-Midpoint Reversal
- priority: Low — The rule is explicit but depends on intraday quote/trade data and has high-turnover execution risk.
- direction: long/short | market: US equities | timeframe: 30-minute bars (intraday-only)
- indicators: Intraday close divided by geometric mean of accumulated daily high and low
- Overview: The intraday version fades stock-specific deviations of the current intraday close from the day's high-low midpoint. It re-estimates the mean-reversion coefficient by time of day and sector.
- Stock or ETF Criteria: US equities with daily OHLC, intraday bars, and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute hlC=intraday_close/sqrt(daily_high*daily_low), winsorize by timestamp, demean by ind1, and apply the fitted coefficient for the current intraday bucket.
- Exit conditions: Forecast-driven positions are updated by qsim; a discrete exit/stop rule is not specified in source.
- Trade management: Intraday qsim engine rebalances forecast positions; no explicit stop/target in the alpha module.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday bars only; daily OHLC cannot recreate the within-session signal.
- reported_metrics: none reported
- notes: Horizon and bar frequency are configurable; default examples use 30-minute bars.
- sources: [noterminusgit/statarb — Intraday high-low signal, regression and forecast generation](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/hl_intra.py#L168-L403)

### G013 | N-Bar Contrarian Sign-Flip (mean of last N returns)
- priority: Low — Daily analog already covered by catalog reversal family; value is the honest cost analysis.
- direction: long/short | market: FX EUR/USD (6h bars) | timeframe: 6h (intraday-adaptable)
- indicators: Mean of the last N bar returns (N=3 primary; optimized 51 on 6h)
- Overview: Simplest possible reversal: if the mean of the last 3 bars' returns is negative go long, positive go short, always in market. Demonstrates both the short-horizon reversal effect and the turnover problem - the notebook's own analysis shows transaction costs eating ~50% of outperformance and 553 trades.
- Stock or ETF Criteria: EUR/USD; any liquid instrument.
- Stocks to avoid and known failure conditions: Always-in-market with N=3 -> extreme turnover, cost-fragile; window sensitivity is severe (N=5 underperformed buy-and-hold).
- Entry conditions: Position = -sign(mean(returns of last N closes)), shifted one bar.
- Exit conditions: Signal flip (always in market).
- Trade management: Unit +/-1 position; proportional cost per position change.
- Target: Short-horizon reversal capture.
- Risk factors and risk management: Turnover dominates; parameter curve-fit risk (N=51 optimal was data-mined).
- Best known candlestick timeframe(s): 6h native; daily version = 3-day reversal (cf. catalog short-term reversal family S027-S031).
- reported_metrics: Notebook: +9.6% vs -7.3% buy-and-hold over 2y sample pre-costs (repo-claimed, unverified).
- notes: Overlap S027-S031 family (short-term reversal).
- sources: [BearsOnMars/Algorithmic_Trading — window=3 sign-flip + cost sensitivity](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Momentum/Momentum%20aka%20Contrarian%20Strategies%20-%20Design%20and%20Backtest.ipynb); [BearsOnMars/Algorithmic_Trading — window=51 optimized](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Momentum/Momentum%20%28Contrarian%29%20Backtester%20Class%20with%20OOP.ipynb)

### G014 | Quote-Based High-Low Multi-Lag Reversion
- priority: Low — The signal family has multiple horizons and explicit market-neutral preprocessing, but its regression and execution data are specialized.
- direction: long/short | market: US equities | timeframe: Daily and intraday quote bars (intraday-adaptable)
- indicators: Quote high/low geometric midpoint, lagged daily deviations, industry demeaning
- Overview: This family combines quote-based daily high-low deviations with intraday quote dislocations and regression-fitted lag weights. The source offers combined, multi-timeframe, and hourly-coefficient variants.
- Stock or ETF Criteria: US equities with quote-based daily high/low, 15-minute intraday bars, and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Construct qhl0=close/sqrt(qhigh*qlow) and lag qhl signals; combine them with the current intraday qhlC signal using regression coefficients. `qhl_multi` sets the current intraday weight to zero; `qhl_both_i` uses six time-of-day coefficients.
- Exit conditions: Forecasts update as the fitted signal changes; no explicit price stop or target is specified.
- Trade management: Regression-fitted coefficients feed the daily/intraday simulation engines; portfolio constraints and fills are external.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily lagged signals can be used with daily bars; combined/intraday variants need intraday quotes.
- parameter_variants: qhl_both uses intraday+daily lags; qhl_multi sets current intraday weight to zero; qhl_both_i fits six hourly buckets.
- reported_metrics: none reported
- notes: These are distinct implementations of the same high-low signal family; the original code's coefficient fit is estimated in sample before middate.
- sources: [noterminusgit/statarb — Combined daily/intraday quote-based factor and fitted forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/qhl_both.py#L119-L407); [noterminusgit/statarb — Multi-lag signal with zero intraday coefficient](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/qhl_multi.py#L144-L747); [noterminusgit/statarb — Time-of-day intraday coefficient variant](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/qhl_both_i.py#L55-L291)

### G015 | Quote-Based Intraday High-Low Reversion
- priority: Low — The signal uses explicit microstructure data but is high-turnover and not suitable for daily-only execution.
- direction: long/short | market: US equities | timeframe: 15-minute bars (intraday-only)
- indicators: Intraday close relative to quote-based daily high/low; six time-of-day regression buckets
- Overview: This strategy uses quote extremes rather than executed-trade extremes to detect intraday dislocations and fits separate coefficients for six hourly periods. It aims to capture liquidity/microstructure-driven reversion.
- Stock or ETF Criteria: US equities with quote-level highs/lows, intraday bars, and Barra industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute qhlC=intraday_close/sqrt(qhigh*qlow), winsorize and industry-demean by timestamp; apply the coefficient fitted for the current hour and the configured forward-return horizon.
- Exit conditions: Forecast positions are updated intraday; the source does not specify a discrete stop or fixed holding period.
- Trade management: The qsim engine handles forecast positions; exact sizing and costs are configured separately.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only; daily bars cannot represent quote-extreme timing.
- reported_metrics: none reported
- notes: Model fits are split by an in-sample middate; lookahead should be checked in the external simulation pipeline.
- sources: [noterminusgit/statarb — Quote-based intraday signal, six hourly buckets and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/qhl_intra.py#L85-L333)

### G016 | RSI Threshold + Higher-High Confirmation Entries (Zerodha Kite, Indian equities)
- priority: Low — The high>prev-high confirmation on RSI entries is a testable micro-filter; otherwise generic.
- direction: long | market: Indian equities via Zerodha | timeframe: intraday bars (intraday-adaptable)
- indicators: RSI; current bar high vs previous bar high; Heikin-Ashi conversion available
- Overview: Live-trading notebooks for Zerodha Kite that buy on RSI conditions - the interesting variant requires the current candle's HIGH to exceed the previous candle's high (momentum-confirmation on the RSI signal rather than blind oversold buys). Heikin-Ashi conversion is included as an alternative bar type.
- Stock or ETF Criteria: NSE equities with Kite API data.
- Stocks to avoid and known failure conditions: Notebook-quality live scripts; poll-loop data handling is fragile.
- Entry conditions: RSI threshold (oversold/bullish) AND current high > previous high (confirmation variant); plain RSI<30 buy / RSI>70 sell in the 'Normal RSI' notebook.
- Exit conditions: RSI-driven sell (SELL on RSI notebook) / limit orders.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday; daily adaptation = RSI + higher-high confirmation.
- reported_metrics: none reported
- notes: Overlap S001/S011 vicinity.
- sources: [ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha — RSI + higher-high entry](https://github.com/ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha/blob/697e091400cf49219248e8c49c0be662157af0f0/Buy%20on%20RSI%20when%20the%20%20current%20high%20of%20candle%20is%20more%20then%20previous%20high%20of%20candle%20.ipynb); [ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha — plain RSI buy/sell](https://github.com/ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha/blob/697e091400cf49219248e8c49c0be662157af0f0/Normal%20%20RSI%20buy_sell%20.ipynb); [ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha — RSI exit](https://github.com/ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha/blob/697e091400cf49219248e8c49c0be662157af0f0/SELL%20on%20RSI%20.ipynb)

### G017 | RSI(14) 30/70 Reversal
- priority: Low — Exact thresholds are given, but RSI(14) 30/70 is a broad tutorial rule without a regime filter or evidence.
- direction: long/short | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: Smoothed 14-session RSI
- Overview: The code maps RSI extremes to contrarian positions, buying oversold readings and shorting overbought readings. It is a basic RSI(14) oscillator rule, not the catalog's RSI(2) family.
- Stock or ETF Criteria: Daily equities with 14 sessions of price history.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Enter long when RSI(14)<30 and short when RSI(14)>70; the signal series switches to the corresponding side.
- Exit conditions: Exit/reverse when the opposite RSI extreme is reached; no separate mean-cross or time stop is specified.
- Trade management: Signal-based long/short exposure; no sizing or costs specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- sources: [je-suis-tm/quant-trading — RSI(14) calculation and 30/70 signal](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/RSI%20Pattern%20Recognition%20backtest.py#L56-L90)

### G018 | RSI+MA Mean-Reversion on EUR/USD M1 (OANDA)
- priority: Low — Intraday-only skeletal bot; nothing transferable beyond the RSI+MA combo.
- direction: long/short | market: FX EUR/USD via OANDA | timeframe: M1 (intraday-only)
- indicators: RSI + moving average on 1-minute candles (per README); fixed pip SL/TP
- Overview: OANDA market-order bot on EUR/USD 1-minute bars using an RSI/MA mean-reversion signal with fixed 20/50-pip stop/target. Minimal skeleton - signal file mostly wraps orders.
- Stock or ETF Criteria: EUR/USD.
- Stocks to avoid and known failure conditions: M1 noise + fixed-pip exits = spread-dominated; signal logic not fully in code.
- Entry conditions: RSI/MA-based signal (documented in README, code mainly wraps orders); market order 1000 units.
- Exit conditions: stopLossOnFill -20 pips, takeProfitOnFill +50 pips.
- Trade management: Fixed 1000 units.
- Target: +50 pips.
- Risk factors and risk management: -20 pip stop.
- Best known candlestick timeframe(s): M1 only.
- reported_metrics: none reported
- notes: Signal implementation incomplete in the tracked code.
- sources: [XBT3K/RSI-MA-EURUSD-Algo — order wrapper + M1 granularity](https://github.com/XBT3K/RSI-MA-EURUSD-Algo/blob/996076f712fe6c92a216134e0848614b2bf45609/mean_reversion_trading.py); [XBT3K/RSI-MA-EURUSD-Algo — strategy description](https://github.com/XBT3K/RSI-MA-EURUSD-Algo/blob/996076f712fe6c92a216134e0848614b2bf45609/README.md)

### G019 | Stochastic-Threshold Mean Reversion on SPY (IB TWS)
- priority: Low — Generic stochastic MR on SPY; overlapping catalog family.
- direction: long | market: SPY via Interactive Brokers | timeframe: intraday (intraday-adaptable)
- indicators: Stochastic oscillator thresholds
- Overview: Early-stage IB/TWS bot: buys SPY when the stochastic falls below an oversold threshold and sells when it returns to high levels - plain stochastic mean reversion on the index.
- Stock or ETF Criteria: SPY.
- Stocks to avoid and known failure conditions: Skeleton project (initial development); no risk layer.
- Entry conditions: Stochastic < oversold threshold -> buy SPY.
- Exit conditions: Stochastic > high threshold -> sell.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday; daily analog exists in catalog (stochastic family).
- reported_metrics: none reported
- notes: Overlap S001/S011 vicinity.
- sources: [tranceitionalMynd/spy-reaper — bot](https://github.com/tranceitionalMynd/spy-reaper/blob/688db6ea8e5f489625df5046900807055a4e1b77/spy-reaper/spy-reaper.py); [tranceitionalMynd/spy-reaper — strategy description](https://github.com/tranceitionalMynd/spy-reaper/blob/688db6ea8e5f489625df5046900807055a4e1b77/README.md)

### G020 | Two-Day Dip with Five-Session Hold
- priority: Low — The rule is short and reproducible but has no regime gate, cost model, or evidence of robust edge.
- direction: long | market: Equities; Backtrader sample | timeframe: 1d (daily-native)
- indicators: Two consecutive lower closes
- Overview: The beginner strategy buys a short two-day price dip and holds for a fixed five sessions, testing short-horizon reversal without additional indicators.
- Stock or ETF Criteria: Example instrument supplied to Backtrader.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Buy when the latest two closes are below their prior closes; the Backtrader code uses the current close and previous close conditions before issuing a market order.
- Exit conditions: Sell after five sessions; orders created at bar close execute on the next session open.
- Trade management: One long position at a time; no sizing, stop, or transaction costs specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: A separate `TestStrategy` in the same file is a buy-and-hold demo and is skipped.
- sources: [yungalyx/NoisyBoyAlgotrader — Two-day dip strategy and five-session exit](https://github.com/yungalyx/NoisyBoyAlgotrader/blob/6c5783b0a8d602c1455e34e9d9eb664edc7b0b43/src/strategies/beginner.py#L37-L85)

### G021 | Volume-Adjusted Daily Return Reversal
- priority: Low — The volume-conditioned reversal thesis is interpretable but depends on proprietary data and has no reported cost-aware validation.
- direction: long/short | market: US equities | timeframe: 1d (daily-native)
- indicators: 21-day relative volume, beta-adjusted daily returns, lagged volume-return interaction
- Overview: Unusual volume scales a stock's beta-adjusted return to identify temporary price/flow imbalances. The multi-period version removes the intraday component and uses raw relative volume times beta-adjusted returns.
- Stock or ETF Criteria: US equities with tradable volume, 21-day median volume, beta and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Set relative volume to tradable_volume / 21-session median; compute beta-adjusted return; daily signal is relative-volume × beta-adjusted return (or, in `vadj.py`, volume-weighted sign of return), then winsorize/demean and fit lagged forecasts.
- Exit conditions: Daily forecast is recomputed; no fixed price stop or exit is specified.
- Trade management: Forecast is passed to bsim; no direct order sizing is defined.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native.
- parameter_variants: vadj.py uses volume times sign of beta-adjusted return and adds intraday features; vadj_multi.py uses relative volume × beta-adjusted return with daily lags.
- reported_metrics: none reported
- notes: The legacy `vadj_old.py` is superseded by current modules and is not recorded as a separate strategy.
- sources: [noterminusgit/statarb — Daily and combined volume-adjusted signal variants](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/vadj.py#L206-L432); [noterminusgit/statarb — Daily-only raw-volume/beta-adjusted-return signal](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/vadj_multi.py#L113-L320)

### G022 | Volume-Adjusted Intraday Reversal and Directional Sizing
- priority: Low — The rules are specified but require intraday execution and are not applicable to daily-only data.
- direction: long/short | market: US equities | timeframe: Intraday bars (intraday-only)
- indicators: Intraday relative volume, beta-adjusted returns or return sign, six time-bucket coefficients
- Overview: The intraday-only model scales beta-adjusted return by unusual volume, while the position-oriented variant uses only the direction of the return and lets volume determine magnitude. Time-of-day coefficients capture different liquidity regimes.
- Stock or ETF Criteria: US equities with intraday price/volume, beta, and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each intraday bar, compute current return (overnight return + log(intraday close/daily open)), subtract beta×market return, then multiply by relative volume; the `vadj_pos.py` variant uses sign(adjusted return)×relative volume.
- Exit conditions: Forecasts update through qsim; discrete exit and stop rules are not specified.
- Trade management: Intraday alpha is consumed by the qsim execution engine.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only.
- parameter_variants: vadj_intra.py uses raw beta-adjusted return; vadj_pos.py uses its sign and volume-driven magnitude.
- reported_metrics: none reported
- notes: The model is an alpha generator, not a complete position lifecycle.
- sources: [noterminusgit/statarb — Intraday-only volume-adjusted signal](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/vadj_intra.py#L121-L260); [noterminusgit/statarb — Sign-based directional signal with volume-controlled magnitude](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/vadj_pos.py#L135-L362)


## Pairs, Spread and Statistical Arbitrage

### G023 | ADF+Hurst+Half-Life Screened Pair Reversion with Bollinger Distance Entry
- priority: High — Complete daily-bar implementable spec with triple statistical screening - Chan-lineage pair construction done properly.
- direction: market-neutral | market: US equities (Yahoo scrape) | timeframe: 1d (daily-native)
- indicators: Correlation>0.75 (0.9 for existing position), ADF cointegration at 1% crit, Hurst<0.5, OU half-life<30 days, 200-day Bollinger bands on each leg
- Overview: Multi-filter pairs pipeline: screen correlated pairs (>0.75), require ADF cointegration at the 1% level, Hurst exponent <0.5 (mean-reverting spread), and OU half-life <30 sessions (reversion must be fast enough). Then short the higher-priced leg and long the lower when price approaches the 200-day Bollinger band and the partner sits on the opposite side of the mean. Stricter screening than a raw z-score pair trade.
- Stock or ETF Criteria: Pairs passing corr>0.75 + ADF(1%) + Hurst<0.5 + half-life<30d. Suggested: same-sector liquid names.
- Stocks to avoid and known failure conditions: Hurst>=0.5 means persistent spread - untradeable; long half-life (>30d) means capital locked too long; strict filters can leave zero tradable pairs.
- Entry conditions: Short leg: last price within `dist` (default 2 points) of its 200d upper Bollinger band while partner < middle band (or the lower-band mirror). Long leg: within dist of lower band while partner > middle. Position: short higher-priced leg, long lower-priced leg.
- Exit conditions: Stop when short leg exceeds upper band (entry_exit_points[0]) or long leg drops below lower band; take profit when each leg reaches its middle band (mean).
- Trade management: One pair at a time via find_best_pair (lowest ADF stat among candidates passing entry/exit price checks); dollar legs implicit.
- Target: Each leg reverting to its 200d mean (middle Bollinger).
- Risk factors and risk management: Leg-level band stops rather than spread stop - leg prices, not spread, drive exits (a modeling weakness: stops are on absolute price, not the spread). Half-life<30d bound.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: README claims $10k -> $23.6k over 5 years, ~27% annualized (repo-claimed, unverified; single-pair simulation, likely no costs).
- external_refs: Ernie Chan, Algorithmic Trading (ADF/Hurst/OU half-life machinery)
- notes: Stops are on individual leg prices not the spread - flag for reimplementation.
- sources: [stephenkyang/mean-reversion-pairs-trading — finding_correlated_pairs/ADF_test/hurst_analysis/half_life/entry_exit_points/find_best_pair](https://github.com/stephenkyang/mean-reversion-pairs-trading/blob/77b3d828615b354240cc3e957d4bbedfdf79ae78/model.py); [stephenkyang/mean-reversion-pairs-trading — methodology + 5y sim claim](https://github.com/stephenkyang/mean-reversion-pairs-trading/blob/77b3d828615b354240cc3e957d4bbedfdf79ae78/README.md)

### G024 | Avellaneda-Lee PCA Eigenportfolio S-Score Reversion (Equity_StatArb implementation)
- priority: High — Published market-neutral methodology with explicit s-score thresholds and daily-bar mechanics; canonical stat-arb backbone.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: PCA of standardized returns; OLS residual regression; AR(1) s-score = -m/sigma_eq
- Overview: Implements Avellaneda & Lee (2010) statistical arbitrage: regress each stock's returns on PCA eigenportfolio factor returns, accumulate residuals, fit AR(1) to get an s-score; long stocks with deeply negative s-score, short positive. Market-neutral cross-sectional mean reversion; the paper historically reported Sharpe ~1.1-1.5 before costs.
- Stock or ETF Criteria: not specified in source. Paper universe: S&P 500 constituents. Suggested: liquid large caps, ~15 PCA factors.
- Stocks to avoid and known failure conditions: Fails when residual autocorrelation is near zero (no AR(1) fit) or factor structure shifts; s-score filter skips stocks with -log(b)*252 <= k (insufficient mean-reversion speed).
- Entry conditions: Compute s-score = -m/sigma_eq per stock after PCA residual AR(1) fit (delay-window PCA, 15-factor coef frame). Paper convention: buy if s < -1.25 (heavy) / -0.75, sell if s > +1.25/+0.75.
- Exit conditions: Close when s-score reverts through ~0 (paper: close bands near +/-0.5 on reversal); hard stop at extreme adverse s per paper.
- Trade management: Long/short book, dollar-neutral weighting by s-score magnitude; cumulative residuals tracked continuously.
- Target: Reversion of cumulative residual to m (AR(1) mean); expected holding days per paper.
- Risk factors and risk management: not specified in source. Suggested: PCA strips factor risk; cap per-name weight; stop if s moves further adverse >2 sigma_eq.
- Best known candlestick timeframe(s): Daily bars (native to paper and code).
- reported_metrics: none reported
- external_refs: Avellaneda & Lee, 'Statistical arbitrage in the US equities market' (2010)
- notes: Same strategy as the Hudson&Thames arbitrage_research 'Avellaneda-Lee PCA Eigenportfolio S-Score Reversion' record - merge sources at assembly.
- sources: [BananaHamm/Equity_StatArb — find_Factor PCA + find_Residue + find_Target_sscore s-score computation](https://github.com/BananaHamm/Equity_StatArb/blob/d62e2d5ea2bf8fe7cf75a88b768ebc9d7f404a93/algorithm.py)

### G025 | Avellaneda-Lee PCA Eigenportfolio S-Score Reversion
- priority: Medium — The portfolio has a clear daily signal and published basis, but code evidence is a short gross demonstration.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: 15 PCA components, 252-session correlation window, 60-session residual window, OU S-score
- Overview: The strategy decomposes a 176-stock universe into principal-component factor returns, fits residuals for each eigenportfolio, and trades stocks whose residual S-score is sufficiently extreme. It is a basket/stat-arb method rather than a two-name pair.
- Stock or ETF Criteria: 176 US stocks in the notebook's sample; factor weights are estimated from the prior 252 sessions.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Fit 15 PCA components on 252 sessions, estimate factor returns/residuals over 60 sessions, and compute S-scores with k=8.4. Open long at S<−1.25 or short at S>1.25; close long at S>0.75 and close short at S<−0.5.
- Exit conditions: Shift target weights forward one day before applying returns; positions close at the S-score thresholds.
- Trade management: Normalize long/short portfolio weights so each side sums to one; target position size is 1.
- Target: Close when the S-score reaches the opposite thresholds (long 0.75, short −0.5).
- Risk factors and risk management: The 2018–2019 example is short and no transaction-cost treatment is specified.
- Best known candlestick timeframe(s): Daily equity returns are native.
- reported_metrics: none reported
- external_refs: Marco Avellaneda and Jeong-Hyun Lee, Statistical Arbitrage in the U.S. Equities Market (2010), as cited by the notebook.
- notes: Uses the residual S-score rule; crossref to the existing broad Chan basket is not made because the PCA eigenportfolio construction is distinct.
- sources: [hudson-and-thames/arbitrage_research — PCA residual/S-score portfolio parameters, entry/exit thresholds and shifted returns; code cells 21–41](https://github.com/hudson-and-thames/arbitrage_research/blob/4340ea15c7ab8d0dbeea55a9af10ffa5e8815211/Other%20Approaches/pca_approach.ipynb)

### G026 | Bollinger-Band EWA/EWC Pair Spread Reversion
- priority: Medium — 
- overlap: S071 (existing catalog) — Uses a fixed EWC/EWA spread with long entry z<−1.5, short entry z>1.8, and closes within ±0.2 of zero. This is a pair-spread variant of S071; the notebook's cointegration tests are unstable over the sample.
- direction:  | market: Canadian/Australian equity ETFs (EWC/EWA) | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [orangepineapple/mean_reversion_strategies — Moving z-score pair entries at −1.5/+1.8 and mean-band exit buffer 0.2](https://github.com/orangepineapple/mean_reversion_strategies/blob/e6d64522861c4a0551c6d423d4b5e95955049922/EWC_vs_EWA_mean_reversion_bollinger_band/backtest_ewc_ewa_pair_trade.py#L18-L128)

### G027 | Chan Cointegrated-Pair Z-Score Backtester (PairTradeR)
- priority: Medium — Standard Chan pair z-score = S071 core rule; notable for OU half-life parameter optimization tooling.
- overlap: S071 (existing catalog) — Same z-score-threshold pair entry as S071; adds rollingZScoreBothThresh long/short helper and OU half-life optimizer for parameter selection.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: rolling z-score of spread; ADF p-value; OU half-life via regression
- Overview: R/Shiny GUI backtesting pair-trading stat-arb per Ernie Chan 'Algorithmic Trading': rolling z-score threshold entries on cointegrated spreads, ADF test for stationarity, OU half-life computed via d_spread ~ spread regression.
- reported_metrics: none reported
- external_refs: Ernie Chan, Algorithmic Trading: Winning Strategies and Their Rationale
- sources: [justinlent/PairTradeR — rollingZScore* helpers, calcADFpvalue, calcOUprocess half-life](https://github.com/justinlent/PairTradeR/blob/1f262a97da0dbf9527457a612a7d35db1759c112/utilityFunctions/zScore.R); [justinlent/PairTradeR — parameter-combo simulation engine](https://github.com/justinlent/PairTradeR/blob/1f262a97da0dbf9527457a612a7d35db1759c112/utilityFunctions/strategySim.R)

### G028 | Cointegrated Pair Spread Reversion with Risk Limits
- priority: Medium — 
- overlap: S071 (existing catalog) — Uses an OLS log-price spread with configurable entry/exit standard deviations (CLI defaults 2.0/0.5) plus a loss limit and borrow-cost option; this is a direct cointegrated-pair z-score variant of S071.
- direction:  | market: Equities | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [wywongbd/pairstrade-fyp-2019 — Log-price cointegration residual, mean/std thresholds and risk exit](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/ptstrategy_cointegration.py#L61-L148); [wywongbd/pairstrade-fyp-2019 — Cointegration strategy parameters and commission/borrow settings](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/backtest_pair.py#L144-L157)

### G029 | Cointegrated Pair Z-Score Mean Reversion
- priority: Medium — 
- overlap: S071 (existing catalog) — Source uses cointegration tests on formation data, OLS hedge ratios, ±2 spread-standard-deviation entries, and closes when the spread returns through its mean; it is a direct generic-pair implementation of S071. Other sources use fixed-beta rolling spread z-scores with differing thresholds (Honoreaa: beta=0.2, entry ±2/exit ±0.5; Simao: default entry/exit level 1.0 and five stabilizing observations). The KidQuant notebook is educational and illustrates cointegrated-pair selection and ratio z-score bands, but it does not specify a complete exit/backtest. Sapphire's HFT notebook uses a 2.5σ entry, 0.5σ exit gap, 5σ stop, and at most one pair on minute data.
- direction:  | market: Equities; sample universe not specified in source | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [financialnoob/pairs_trading — Engle–Granger/Johansen pair selection, ±2-standard-deviation spread entries and mean-reversion exit; code cells 86–95](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/15.coint_based_stat_arb.ipynb); [financialnoob/pairs_trading — Standard cointegration baseline with ±2σ spread triggers and zero-cross exits; code cells 23–31](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/20.pairs_trading_with_pci/20.pairs_trading_with_partial_cointegration.ipynb); [honoreaa/Futures-MeanReversion — Rolling spread z-score with pair signal thresholds](https://github.com/honoreaa/Futures-MeanReversion/blob/673a22017225c7eb250e6812acc615070d8ea6ac/src/signal_logic.py#L3-L17); [honoreaa/Futures-MeanReversion — ES/NQ example uses beta=0.2, 30-session z-score, entry ±2 and exit ±0.5; code cells 3 and 6](https://github.com/honoreaa/Futures-MeanReversion/blob/673a22017225c7eb250e6812acc615070d8ea6ac/notebooks/3_zscore.ipynb); [simaomsarmento/PairsTrading — Fixed-beta normalized spread threshold strategy, default entry/exit levels 1.0 and five-session stabilization](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/classes/class_Trader.py#L21-L70); [KidQuant/Pairs-Trading-With-Python — Engle-Granger p<0.05 pair screen, OLS spread and rolling 5/20 ratio z-score examples; code cells 30, 38, 42, and 48](https://github.com/KidQuant/Pairs-Trading-With-Python/blob/8e86c88272e5e0aec6970fb3b95dc7f39813702f/PairsTrading.ipynb); [sapphire921/HFT-Pairs-Trading — Cointegration screen, hedge-ratio spread and intraday z-score thresholds; code cells 2–6](https://github.com/sapphire921/HFT-Pairs-Trading/blob/2048b7cf3352d3db83236e6dfcccc47e28596c06/Code.ipynb)

### G030 | Commodity Crack-Spread Cointegration Reversion
- priority: Medium — 
- overlap: S071 (existing catalog) — Applies cointegrated-spread reversion to 3:2:1, 1:1:0 gasoline, and 1:0:1 heating-oil crack spreads. Calls specify `ma_lookback=5`, `entry_sigma=2`, and contract-ratio weights; the internal exit rule is private.
- direction:  | market: Crude oil, gasoline, and heating-oil futures | timeframe: Daily futures (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [rolling-panda-san/notebooks — Cointegration tests and GP1999 crack-spread calls with MA lookback 5 and entry sigma 2; code cells 22, 45, 49, and 53](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_crack_spread_stat_arb.ipynb)

### G031 | Crude-Oil Calendar-Spread Bollinger Mean Reversion (CL front vs second month)
- priority: Medium — Genuine microstructure-aware futures RV edge with honest bid/ask modeling, but intraday-native - daily adaptation unproven.
- direction: long/short | market: NYMEX CL futures calendar spread | timeframe: 1-minute bid/ask (intraday-adaptable)
- indicators: 60-period rolling mean +/- 2 std Bollinger Bands on the front-minus-second-month spread, built from bid/ask quotes
- Overview: Trades the crude-oil front-month vs second-month calendar spread mean-reverting around a 60-minute Bollinger band: buy the spread (long front, short second) below the lower band, sell above the upper. Calendar spreads are highly mean-reverting because both legs share the underlying - a classic futures relative-value edge. Realistic microstructure: separate bid/ask bands for buys and sells, option to pay the spread vs midpoint fill.
- Stock or ETF Criteria: CL futures front + second month (CONTRACT_NUMS=1,2); liquid NYMEX session; needs bid/ask quote data.
- Stocks to avoid and known failure conditions: Avoid around contract roll/expiry when front-month microstructure decouples; FILL_AT_MIDPOINT=False is the honest cost model (pay the spread). Mean reversion fails during storage/supply shocks that reprice the curve durably.
- Entry conditions: Buy spread when spreads_for_buys (front ask - back bid) < lower band (60-bar rolling mean - 2 std); sell spread when spreads_for_sells (front bid - back ask) > upper band. Positions = weights shifted one bar.
- Exit conditions: Signal flips to opposite band or spread reverts inside the band; exits implicit in target-weights sign change.
- Trade management: Unit position on the spread; positions shifted 1 bar to avoid lookahead. Commission: $0.85 broker + $1.50+0.02 exchange per contract (NymexCommission).
- Target: Reversion toward the 60-bar rolling mean of the spread.
- Risk factors and risk management: Commissions/fees are large relative to spread ticks; a structural break in the curve (e.g., 2020 negative front-month) is the tail risk.
- Best known candlestick timeframe(s): Original: 1-minute bid/ask bars. Daily adaptation: use daily settlement spread of CL1-CL2 with 60-day bands; plausible but weaker edge (intraday liquidity dislocations vanish).
- reported_metrics: Backtest window 2019-01 to 2019-08 run via Moonshot (repo-claimed, unverified).
- external_refs: Ernie Chan-style futures calendar-spread stat arb (QuantRocket codeload example)
- sources: [quantrocket-codeload/calspread — CalendarSpreadStrategy: dual bid/ask Bollinger bands, NymexCommission](https://github.com/quantrocket-codeload/calspread/blob/8276f033b705040d1fe6d173d2ffa4620d30ac5f/calspread/calspread.py); [quantrocket-codeload/calspread — backtest invocation](https://github.com/quantrocket-codeload/calspread/blob/8276f033b705040d1fe6d173d2ffa4620d30ac5f/calspread/Part3-Moonshot-Strategy.ipynb)

### G032 | Crypto Pair Z-Score Reversion (BTC/ETH, Johansen screening)
- priority: Medium — Crossref to S071 mechanics with crypto universe; stablecoin pairs are screened out via cointegration.
- overlap: S071 (existing catalog) — Johansen multi-asset screening added; z-score thresholds +/-1 (vs catalog 2).
- direction: market-neutral | market: crypto spot (Yahoo: BTC,ETH,BNB,SOL,DOGE,XRP,LUNA,SHIB + stables) | timeframe: 1d (daily-native)
- indicators: Engle-Granger & Johansen cointegration, spread z-score, price ratio
- Overview: Notebook screens ~10 crypto tickers for cointegrated pairs (Engle-Granger + Johansen), then trades the BTC-ETH spread on z-score thresholds (+/-1 plotted bands). Same z-score pair mechanics as S071 applied to crypto.
- reported_metrics: none reported
- notes: Includes stablecoins in screening; LUNA1/SHIB data issues make the chosen pair effectively BTC/ETH.
- sources: [eshan-kaul/PairsTrading-Crypto — Johansen screening + z-score spread bands +/-1](https://github.com/eshan-kaul/PairsTrading-Crypto/blob/6ef40076b7bc2a4db6f491e24058a0c9211868e6/Pairs%20Trading%20Strategy%20for%20Crypto%20Markets.ipynb)

### G033 | Decorrelation-Gated Buy-the-Loser Pair Rotation (PEP/KO example)
- priority: Medium — Distinctive long-only pairs mechanic - correlation-gate + buy-the-loser - testable on daily bars; simpler than full stat arb.
- direction: long | market: equity pairs (PEP/KO example) | timeframe: 1d (daily-native)
- indicators: Rolling 9-day correlation of the pair's 20-day returns; prior-day return per leg
- Overview: Long-only daily pair rotation: each day, buy whichever leg had the LOWER prior-day return - but only trade when the rolling 9-session correlation of the legs' 20-day returns is <0.9, i.e., only when the pair is currently decorrelated/diverging. The thesis: a highly-correlated pair that has just decorrelated will recouple, so buying the recent loser captures the reconvergence. Trades daily with a hold-loser rule.
- Stock or ETF Criteria: Highly-correlated equity pairs (PEP/KO demo); needs sustained structural correlation.
- Stocks to avoid and known failure conditions: When correlation never drops below 0.9 there are no signals; buying losers unhedged (no short leg) leaves full market exposure; correlation computed on 9 obs of 20d returns is noisy.
- Entry conditions: If rolling-9d corr of 20d returns < 0.9: long the leg with the lower prior-day return; keep holding yesterday's losing position until profitable.
- Exit conditions: Switch to the other leg when it becomes the lower performer and correlation gate still passes; hold-loser rule keeps unprofitable positions.
- Trade management: Single long leg at a time, swaps daily; no short leg.
- Target: Reconvergence of the decorrelated pair / loser catching up.
- Risk factors and risk management: Market-beta exposure (long-only); divergence can persist (correlation stays <0.9 while spread keeps widening).
- Best known candlestick timeframe(s): Daily bars (native).
- reported_metrics: none reported
- notes: Unhedged long-only - unlike S071; closer to a relative-strength rotation between pair legs.
- sources: [LongOnly/Quantitative-Notebooks — correlation<0.9 gate + min-return leg selection + hold-loser](https://github.com/LongOnly/Quantitative-Notebooks/blob/9041f80c21b4029afdbd044c60f4b8d85473b70d/PairsTrading.ipynb)

### G034 | Kalman Forecast-Error Pair Reversion
- priority: Medium — The method is explicit, sequential, and daily-adaptable, but the example is one pair and lacks cost/OOS evidence.
- direction: market-neutral | market: EWC/EWA Canadian/Australian equity ETFs | timeframe: 1d (daily-native)
- indicators: Kalman hedge ratio/intercept, forecast error and forecast-error standard deviation
- Overview: A Kalman filter updates the hedge ratio and intercept as each pair price observation arrives; forecast errors are converted to target pair quantities using standard-deviation thresholds. This is a dynamic hedge-ratio alternative to fixed OLS pairs.
- Stock or ETF Criteria: EWC/EWA pair in the example; pair universe is user-supplied.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Initialize the filter with observation covariance 0.001 and transition covariance 0.0001/(1−0.0001); generate trading signals with entry and exit standard-deviation scores both set to 1.
- Exit conditions: Use the Kalman strategy's target quantity to close/reverse pair positions when the forecast-error signal exits; the notebook shifts target quantities by one observation before returns to avoid lookahead.
- Trade management: Hedge-ratio weights are used for the long/short legs; signal quantities are applied one observation later.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The notebook uses a single selected ETF pair and does not show a transaction-cost model.
- Best known candlestick timeframe(s): Daily data in the example; Kalman updates can be applied at other frequencies.
- reported_metrics: none reported
- external_refs: Ernest P. Chan, Algorithmic Trading: Winning Strategies and Their Rationale, as cited in the notebook.
- notes: The notebook is based on Chan-style Kalman pairs; code feeds observations sequentially to avoid future-data use.
- sources: [hudson-and-thames/arbitrage_research — Kalman parameter setup, EWC/EWA signal generation and one-observation execution shift; code cells 15, 33, and 41](https://github.com/hudson-and-thames/arbitrage_research/blob/4340ea15c7ab8d0dbeea55a9af10ffa5e8815211/Other%20Approaches/kalman_filter.ipynb)

### G035 | Kalman-Updated Hedge-Ratio Pair Z-Score Strategy
- priority: Medium — The dynamic hedge ratio and risk-managed pair execution are explicit, but performance depends on pair-specific calibration and is not independently verified.
- direction: market-neutral | market: US equities | timeframe: Daily lookback; bar-frequency position updates (daily-native)
- indicators: Kalman hedge ratio, OHLC-average/log spread, rolling mean and standard deviation, optional Hurst/ADF tests
- Overview: A Kalman filter updates the hedge ratio from daily pair prices; the strategy forms a spread and standardizes the latest bar against lookback mean/std. Optional Hurst and ADF gates reject non-mean-reverting pairs, and the broker risk manager sizes both legs.
- Stock or ETF Criteria: Configured US equity pairs; optional Hurst/ADF filters and optional earnings cease-fire.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: With pair-specific `entryZScore`, enter short when z≥entryZScore and long when z≤−entryZScore. Example configured pairs use lookbacks 10–120 bars and entry thresholds 0.5–2.0; examples generally use log prices.
- Exit conditions: For a long, close when z≥−exitZScore; for a short, close when z≤exitZScore. Example configs omit exitZScore, leaving the parameter default at 0.0 (mean crossing).
- Trade management: RiskManager chooses leg-0 quantity and leg-1 quantity to approximate dollar neutrality using the latest Kalman hedge ratio; market or limit orders are selected by entry/exit distance settings.
- Target: Spread mean crossing by default; configuration examples set `exitZScore=0.0`.
- Risk factors and risk management: Kalman beta, spread scale, and pair-specific z thresholds are estimated/configured from historical data; earnings handling and risk limits are external to the alpha rule.
- Best known candlestick timeframe(s): Daily bars are used for lookback parameter estimation; the same state machine can update on the configured bar frequency.
- parameter_variants: Per-pair lookback 10–120 and entry z-score 0.5–2.0; optional log prices, Hurst and ADF gates; optional limit entry/exit orders.
- reported_metrics: none reported
- notes: The Hurst and ADF flags can overwrite `tradeAllowed` if both are enabled; most sample config rows disable them.
- sources: [tibkiss/huba-v1 — Kalman hedge updates, spread z-score and entry/exit state machine](https://github.com/tibkiss/huba-v1/blob/3996037cb496807c01eb58d2f9ea8a9afac8bf35/strategy/statarb.py#L254-L337); [tibkiss/huba-v1 — Risk-sized two-leg entry and order handling](https://github.com/tibkiss/huba-v1/blob/3996037cb496807c01eb58d2f9ea8a9afac8bf35/strategy/statarb.py#L360-L465); [tibkiss/huba-v1 — Z-score trigger dispatch](https://github.com/tibkiss/huba-v1/blob/3996037cb496807c01eb58d2f9ea8a9afac8bf35/strategy/statarb.py#L509-L519); [tibkiss/huba-v1 — Pair-specific lookback, entry z-score and log-price examples](https://github.com/tibkiss/huba-v1/blob/3996037cb496807c01eb58d2f9ea8a9afac8bf35/config.py#L50-L110)

### G036 | Ornstein–Uhlenbeck Pair-Spread Z-Score
- priority: Medium — 
- overlap: S071 (existing catalog) — Uses OU-derived scores with asymmetric entry/exit thresholds: enter at score >1.25 or <−1.25; close shorts below 0.75 and longs above −0.5. Several notebooks extend the pair universe with ETFs/PCA but retain the same core threshold rule.
- direction:  | market: Equities and ETF baskets | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [financialnoob/pairs_trading — OU-score pair signals and equal-weight long/short positions; code cells 10–11](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/8.pairs_trading_with_ou_process_part1.ipynb); [financialnoob/pairs_trading — The same OU entry/exit rule with additional risk-factor/PCA experiments; code cells 4, 22, 25, 28, and 33](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/9.pairs_trading_with_ou_process_part2.ipynb)

### G037 | Partial-Cointegration Kalman Residual Reversion
- priority: Medium — The random-walk/mean-reversion decomposition is a distinct statistical-arbitrage model with explicit signals, but costs and independent validation are absent.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Partial cointegration model, Kalman-separated mean-reverting component, residual spread
- Overview: The partial-cointegration model separates a stationary mean-reverting component from a random-walk component in the pair spread. Only pairs whose partial model beats AR and random-walk alternatives are traded, with a Kalman estimate of the mean-reverting component supplying the signal.
- Stock or ETF Criteria: Pairs with PAR preferred by AIC, rho in (0.5,0.8), R²>0.5, and beta between 0.75 and 1.25.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Fit the PAR/AR/RW models; keep pairs where PAR has minimum AIC, rho>0.5, R²>0.5, 0.75<beta<1.25, and rho<0.8. Estimate the mean-reverting component M with the Kalman filter; enter short when M>sigma_M and long when M<−sigma_M.
- Exit conditions: Close shorts when M<−0.5·sigma_M and longs when M>0.5·sigma_M; forward-fill positions between signals.
- Trade management: Hedge weights use the fitted beta; source backtests hold equal pair capital and use lagged positions for returns.
- Target: Mean-reverting component returns to zero.
- Risk factors and risk management: The model screens many pairs and parameters in-sample; transaction costs are not specified in the notebook.
- Best known candlestick timeframe(s): Daily bars are native; model parameters are fitted on the training segment.
- reported_metrics: none reported
- external_refs: Clegg and Krauss, Partial Cointegration (2018), as referenced in notebook material.
- notes: The notebook compares the PAR strategy with a standard cointegration baseline; this record covers only the novel partial-cointegration signal.
- sources: [financialnoob/pairs_trading — PAR model fitting, pair screens, Kalman component and ±1σ/0.5σ position rules; code cells 79–91](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/20.pairs_trading_with_pci/20.pairs_trading_with_partial_cointegration.ipynb)

### G038 | ADF Cointegration Screener over Vanguard ETFs
- priority: Low — Pair discovery tool only - no executable entry/exit.
- overlap: S071 (existing catalog) — Screening only; no trade rules. Useful for generating the pair universe for S071.
- direction: market-neutral | market: Vanguard ETFs (74 as of 2017) | timeframe: 1d (daily-native)
- indicators: ADF test on ETF price series over 1-year window
- Overview: Widget that runs Augmented Dickey-Fuller tests across all listed Vanguard ETF pairs on a 1-year window to surface cointegrated pairs for trading. A screening tool for S071-style pair trading rather than a complete strategy.
- reported_metrics: none reported
- sources: [garvit-kudesia91/Statistical-Arbitrage — ADF pair screening widget](https://github.com/garvit-kudesia91/Statistical-Arbitrage/blob/1af3cedff1c009b09fe65ebbbca65d7a5477edf9/Widget%20for%20Co-integration%20check%20%28Vanguard%20ETFs%29.ipynb)

### G039 | Barra Idiosyncratic Residual Return Reversal
- priority: Low — The factor has a clear idiosyncratic-reversion thesis but requires proprietary factor data and a large backtest pipeline.
- direction: market-neutral | market: US equities | timeframe: Daily and intraday (intraday-adaptable)
- indicators: Barra factor-model residual returns, winsorization, multi-day lags
- Overview: The RRB factor takes stock-specific returns left after a Barra risk model, then fits lagged residuals to forward returns. It is a theory-driven idiosyncratic mean-reversion signal distinct from PCA residuals.
- Stock or ETF Criteria: US equities with full Barra exposures and daily/intraday residual-return data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Calculate Barra residual returns, winsorize and demean by date, create lagged daily/intraday residual features, then apply fitted regression coefficients.
- Exit conditions: Forecast alpha through the daily/intraday engines; no explicit single-name exit is specified.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily component is native; intraday version requires 15-minute data.
- reported_metrics: none reported
- notes: Module comments state that Energy is excluded; this is not a complete trading strategy without the portfolio optimizer.
- sources: [noterminusgit/statarb — Barra residual signal, daily/intraday transforms, and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/rrb.py#L56-L236)

### G040 | Bayesian-Optimized OU Half-Life Pair Reversion
- priority: Low — The parameterization is precise and ties exits to estimated mean-reversion speed, but optimization and evaluation reuse the same period.
- direction: market-neutral | market: US equities; example pair MSFT/ACN | timeframe: 1d (daily-native)
- indicators: Log-price spread, rolling z-score, OU kappa and half-life
- Overview: The system screens pairs for cointegration and stationarity, estimates mean-reversion speed/half-life, and trades spread extremes with a stop and half-life-scaled time limit. Gaussian-process Bayesian optimization tunes the spread thresholds.
- Stock or ETF Criteria: Pair must pass Engle–Granger and ADF p<0.05; highest correlation among qualifying pairs is chosen.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Select the highest-correlation pair among those with cointegration and ADF p-values below 0.05. With a 20-session lookback, enter long the spread below rolling mean−1.5σ or short above mean+1.5σ; the pair is MSFT/ACN in the example configuration.
- Exit conditions: Exit on spread reversion through its rolling mean, a stop at 2×rolling spread standard deviation from entry, or after holding_time_factor×half_life (default factor 1.5, half-life 12.25 sessions in the example).
- Trade management: Backtrader trades opposing legs; example initial capital $100,000 and commission 0.1%. Bayesian search runs 50 evaluations over lookback 5–30, entry 1.5–3.0σ, stop 1.5–3.0σ, and holding factor 1.0–2.0.
- Target: Exit at the spread mean; max holding time is tied to estimated half-life.
- Risk factors and risk management: Bayesian optimization maximizes Sharpe and the notebook then backtests the selected parameters on the same 2020–2023 sample, risking substantial in-sample overfit.
- Best known candlestick timeframe(s): Daily bars; default holding time is roughly 18 sessions based on 1.5×12.25-day half-life.
- reported_metrics: none reported
- notes: The source uses a simple log-price ratio spread rather than an OLS hedge ratio; the example pair and optimized parameters are not point-in-time validated.
- sources: [theanh97/Statistical-Arbitrage-Bayesian-Optimized-Kappa-Half-life-Pairs-Trading-Engine — Pair screening, kappa/half-life estimation, Backtrader entry/exit logic, and Bayesian parameter search; code cells 4, 7, 10, 14, 17, and 19](https://github.com/theanh97/Statistical-Arbitrage-Bayesian-Optimized-Kappa-Half-life-Pairs-Trading-Engine/blob/535030aa2d11314a28714a7e6ac9974daa422bab/main.ipynb)

### G041 | Commodity Basis Reversal: Outright and Calendar-Spread Variants
- priority: Low — The basis-reversal family is explicit at the notebook-call level but exact thresholds and cost assumptions are unavailable.
- direction: long/short | market: Commodity futures | timeframe: Daily futures data (daily-native)
- indicators: Front-versus-back contract basis
- Overview: The notebook compares cross-sectional and time-series reversals in commodity futures basis, both as outright contracts and first/second-expiry spreads. The thesis is that extreme near-term basis moves normalize.
- Stock or ETF Criteria: Rossi 2025 commodity futures universe; first/second expiries rolled every 21 sessions.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `XSBasisReversal` or `BasisReversal` on the Rossi 2025 futures universe with front-expiry shift −1 and back shift 0; spread variants trade the first/second expiry legs. Signal thresholds and rankings are private-library details.
- Exit conditions: Recompute the basis-reversal signal; exact exit threshold is not specified in source.
- Trade management: Delta-one futures exposure; weighting differs between outright and spread variants.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Internal signal rules are private and the plotted equity curves are gross of costs.
- Best known candlestick timeframe(s): Daily futures bars; notebook assesses long histories and post-publication results.
- parameter_variants: Cross-sectional outright, cross-sectional spread, time-series outright, time-series spread; front shift −1/back shift 0.
- reported_metrics: none reported
- external_refs: Rossi (2025), as named in the notebook.
- notes: The repository's `vivace` implementation is proprietary; no unobserved spread thresholds are inferred.
- sources: [rolling-panda-san/notebooks — Cross-sectional/time-series outright/spread variants; code cells 6, 11, 16, and 26](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_basis_reversal.ipynb)

### G042 | Commodity Futures Intra-Curve F3/F0 Steepener
- priority: Low — The position is precisely defined but has no timing signal and the source describes unstable seasonal risk.
- direction: long/short | market: Commodity futures | timeframe: Daily futures (daily-native)
- indicators: Third-forward contract versus front contract
- Overview: The notebook constructs a fixed calendar spread that is long the third-forward contract and short the front contract across a commodity universe. It is a curve-steepener exposure rather than a timed mean-reversion entry.
- Stock or ETF Criteria: Commodity futures in the LFIS 2015 universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each commodity, hold +1 exposure to forwardness 3 and −1 to forwardness 0; contracts roll on the fifth business day at a five-session roll period.
- Exit conditions: No signal-based exit is specified; maintain the calendar spread and roll contracts.
- Trade management: Equal-weight across commodities; fifth-business-day roll schedule.
- Target: No fixed target; exposure remains a futures curve steepener.
- Risk factors and risk management: Notebook reports seasonal volatility/drawdowns in natural gas and corn; the unconditional curve exposure can be highly volatile.
- Best known candlestick timeframe(s): Daily futures bars; source highlights winter/spring seasonal risk.
- reported_metrics: none reported
- external_refs: La Française Group (2015), as named in the repository README.
- notes: This is a fixed curve-spread exposure, not a short-term signal-triggered entry.
- sources: [rolling-panda-san/notebooks — F3 long/F0 short construction and equal-weight backtest; code cells 7–8](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_intra_curve.ipynb)

### G043 | Copula Conditional-Probability Pair Trading
- priority: Low — Copula dependence is a distinctive pair-selection signal, but the sample and cost evidence are limited.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: Student-t marginals, fitted Archimedean/Gaussian copulas, conditional return probabilities
- Overview: The strategy fits marginal return distributions and copulas on a formation period, then uses conditional probabilities to identify relative tail events between pair legs. This models nonlinear dependence rather than assuming a Gaussian spread.
- Stock or ETF Criteria: Up to 25 non-overlapping equity pairs with dependence fitted only on the formation sample.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Select up to 25 non-overlapping pairs by descending Kendall tau from the formation period (2018-11-20 to 2020-05-19); fit Student-t marginals and choose the minimum-AIC model from Gaussian, Clayton, Gumbel, Frank, and Joe copulas. With confidence level cl=0.99, go long [1,−1] if conditional CDFs are p1<0.01 and p2>0.99; reverse for p1>0.99 and p2<0.01.
- Exit conditions: Close a long when p1>0.5 or p2<0.5; close a short when p1<0.5 or p2>0.5. The notebook also tests earlier exits at 0.3/0.7.
- Trade management: Positions use equal-weight long/short legs; the notebook doubles return scaling for short capital and does not state a transaction-cost model.
- Target: Mean reversion toward the conditional-probability center.
- Risk factors and risk management: The trading period is six months and pair selection is sensitive to a long formation window; no realistic execution-cost model is specified.
- Best known candlestick timeframe(s): Daily return bars are native; the code's train/test periods are calendar-year scale.
- reported_metrics: none reported
- notes: The exact entry cutoff in the copula loop should be checked before implementation; source code shown here specifies the conditional-probability exit more clearly.
- sources: [financialnoob/pairs_trading — Formation/trading split, Kendall-tau pair selection, copula fitting, and conditional-probability exits; code cells 3, 9, 18, 22, and 30](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/11.pairs_trading_copula.ipynb)

### G044 | Daily PCA Idiosyncratic-Residual Reversal
- priority: Low — The residual-reversion thesis is clear and daily-compatible, but the PCA calibration and portfolio costs require full data dependencies.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: Rolling PCA of equity returns, Barra or PCA residuals, five principal components
- Overview: The model removes common principal-component movements from stock returns and forecasts idiosyncratic residual reversion. A separate daily generator uses exponentially weighted correlations and four PCs.
- Stock or ETF Criteria: US equities with daily returns; PCA generator documentation references a broad liquid universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Fit PCA to a rolling 30-session return window using five components; compute residual actual−predicted returns, winsorize/demean and use fitted lag coefficients. The daily generator variant uses EWM correlation with five-session half-life and four PCs.
- Exit conditions: Daily forecasts feed the optimizer; no explicit single-name stop or target is specified.
- Trade management: Market-neutral forecasts are consumed by the portfolio optimization and execution engines.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily data are native.
- parameter_variants: pca.py uses five PCs/30-session rolling window; pca_generator_daily.py uses four PCs/EWM correlation with five-session half-life.
- reported_metrics: none reported
- notes: PCA decomposition and Barra residual betting are separate in the repository; this record covers PCA-based residuals.
- sources: [noterminusgit/statarb — Rolling PCA, residual generation, and regression forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/pca.py#L57-L464); [noterminusgit/statarb — Daily EWM-correlation PCA residual signal](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/pca_generator_daily.py#L58-L135)

### G045 | Distance-Selected Pair Spread Reversion
- priority: Low — The distance method has complete spread entry/exit logic, but sample-specific selection and costs limit confidence.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Normalized price-distance selection and rolling spread mean/standard deviation
- Overview: The strategy selects pairs by distance between normalized price paths and trades deviations from the rolling spread mean. It includes a profit/loss limit and optional borrow-cost handling.
- Stock or ETF Criteria: Pairs selected from Bloomberg/history data; normalized prices are used for the formation period.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use the configured lookback to calculate spread mean/std; short above mean + entry_threshold×SD and long below mean − entry_threshold×SD. Defaults are set by the backtest CLI, with the common batch run using entry=2 SD.
- Exit conditions: Close a short below mean + exit_threshold×SD and a long above mean − exit_threshold×SD; exit also if trade return falls below `loss_limit` or the short leg value is nonpositive.
- Trade management: Backtrader two-leg positions; borrow-cost modeling is enabled, commission is disabled in the distance variant.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The training/grid-search sample and parameter selection can overfit; no full execution-cost evidence is provided.
- Best known candlestick timeframe(s): Daily data are used.
- reported_metrics: none reported
- notes: The repository compares distance, cointegration and Kalman variants; only the dedicated distance rule is described here.
- sources: [wywongbd/pairstrade-fyp-2019 — Spread levels, ±entry/exit thresholds, and loss breaker](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/ptstrategy_distance.py#L55-L136); [wywongbd/pairstrade-fyp-2019 — Distance/coint/Kalman strategy selection and default threshold arguments](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/backtest_pair.py#L40-L70)

### G046 | Dynamic Vasicek Pair Reversion with Market-Factor Adjustment
- priority: Low — The factor-adjusted state-space formulation is a distinct statistical-arbitrage method, but its position rule remains long-biased when below theta.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Johansen cointegration, EM-fitted Vasicek parameters, market return factor
- Overview: The notebook estimates mean-reverting pair-spread dynamics with a market-factor component and uses the fitted equilibrium level to orient pair exposure. It is distinct from a static z-score because persistence and systematic-market effects are estimated dynamically.
- Stock or ETF Criteria: Non-overlapping equity pairs with Johansen rank evidence; 500-session rolling model window.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Form pairs with Johansen cointegration at the 95% level; estimate EM state parameters on a rolling 500-session window with max lag P=5 and up to 1,000 iterations. Compute the market-adjusted five-session cumulative spread; short when it exceeds theta=B/(1−A), otherwise hold long.
- Exit conditions: The cited test loop recomputes the position each date; no flat-zone or explicit exit threshold is specified.
- Trade management: Long/short leg weights are based on the selected pair; transaction costs are not specified in the cited notebook cells.
- Target: Model-estimated equilibrium theta.
- Risk factors and risk management: The code recomputes an EM fit for each date and pair; this is computationally heavy and no cost-aware OOS validation is reported.
- Best known candlestick timeframe(s): Daily data are native.
- reported_metrics: none reported
- notes: The main algorithm assigns a short only when adjusted spread exceeds theta; otherwise it assigns long.
- sources: [financialnoob/pairs_trading — Johansen screen, EM/Vasicek parameter fit, market-adjusted spread, and position assignment; code cells 15–23 and 33](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/16.pairs_trading_df2006.ipynb)

### G047 | Eight-Session Bollinger Band Pair Reversion
- priority: Low — The entry and exit are explicit but the sample is educational and the backtest is surface-level.
- direction: market-neutral | market: HMC/HYMTF equity pair | timeframe: 1d (daily-native)
- indicators: 8-session rolling mean/std of the HMC–HYMTF price spread
- Overview: The notebook uses a short-window Bollinger channel on a cointegrated stock pair and trades the spread back to its rolling mean. It is a concrete pair portfolio rather than a generic indicator tutorial.
- Stock or ETF Criteria: HMC/HYMTF pair selected by Engle–Granger and ADF tests; example uses a 10-year sample.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Fit the pair using the training period; compute an 8-session spread mean/std. Long the lower spread band at mean−0.8σ and short the upper band at mean+0.8σ; allocate $50,000 to each leg.
- Exit conditions: Close a lower-band long when spread rises above the mean and close an upper-band short when spread falls below the mean.
- Trade management: Dollar allocation is $50,000 per leg; signals are applied on the following day.
- Target: Spread returns to its rolling mean.
- Risk factors and risk management: The notebook uses a short 8-session window and does not account for transaction costs or borrow availability.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The notebook cautions that the research and backtest are preliminary.
- sources: [sap215/StatArbPairsTrading — HMC/HYMTF pair selection, 8-session bands, positions and P&L; code cells 18 and 30–34](https://github.com/sap215/StatArbPairsTrading/blob/46ad070f473fd9f0826aa940fc2357e02734c483/StatArbBlog.ipynb)

### G048 | Granger-Causality-Gated Cointegrated Pair Reversion
- priority: Low — The code's causal screen and trade rules are explicit, but the notebook's in-sample ranking risks selection bias.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Engle–Granger/Johansen cointegration, Granger causality, OLS spread
- Overview: The notebook screens cointegrated pairs for directional Granger predictability and then trades a residual spread. The causal filter adds a pair-selection condition to an otherwise conventional spread-reversion rule.
- Stock or ETF Criteria: Cointegrated, positive-beta equity pairs with directional Granger p-values below 0.05.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Retain pairs with positive OLS beta, ADF/Johansen cointegration flags, and at least one direction with Granger SSR test p<0.05 over lags 1–15. Enter short above +2 spread standard deviations or long below −2.
- Exit conditions: Exit a short when spread falls below zero and a long when spread rises above zero; selected pairs may be ranked by in-sample Sharpe and limited to ten non-overlapping pairs.
- Trade management: Equal-weight pair returns; the selected-pair variant caps the universe at 10 non-overlapping pairs.
- Target: Spread mean crossing at zero.
- Risk factors and risk management: Pair ranking uses in-sample Sharpe; no transaction costs are specified.
- Best known candlestick timeframe(s): Daily price series are native.
- reported_metrics: none reported
- notes: The same notebook also applies wavelet preprocessing; that signal variant is listed separately.
- sources: [financialnoob/pairs_trading — Cointegration/Granger screens, ±2σ entries and zero-cross exits; code cells 86–114 and 127–146](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/14.pairs_trading_granger_causality.ipynb)

### G049 | Intraday PCA Residual Mean Reversion
- priority: Low — The factor is explicit but depends on high-frequency covariance estimates and realistic execution.
- direction: market-neutral | market: US equities | timeframe: 5-minute or 15-minute bars (intraday-only)
- indicators: Rolling intraday return correlations and top-four PCA components
- Overview: The signal removes systematic intraday co-movement with PCA and treats stock-specific residual returns as mean-reverting. It is an intraday version of the daily PCA residual model.
- Stock or ETF Criteria: US equities with intraday bars; top 1,200 by market cap according to the module docstring.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Calculate 5/15-minute bar returns, estimate a rolling 10-period correlation matrix, retain four components, and use actual-minus-reconstructed residual returns as the forecast.
- Exit conditions: Forecast updates intraday; exact exit/holding rule is delegated to the order simulator.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only; daily bars cannot recreate the rolling intraday residual.
- reported_metrics: none reported
- notes: The source uses minimum 10 observations and does not provide a standalone stop or trade policy.
- sources: [noterminusgit/statarb — Intraday correlation/PCA residual generation](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/pca_generator.py#L58-L124)

### G050 | Kagi-Filtered Pair Spread Trend/Reversal Switch
- priority: Low — The Kagi regime switch is a distinct pair-trading rule, but parameter selection and execution assumptions are incomplete.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: 252-session log-price spread, Kagi turning points, spread high-volatility measure
- Overview: Kagi turning points classify the latest pair-spread move, while a spread-volatility ratio selects trend following or contrarian positioning. This is a price-action regime switch applied to pair spreads.
- Stock or ETF Criteria: Pairs selected by the notebook's formation tests; only selected stocks are traded.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each pair, build a 252-session log-price spread and Kagi series using H from the selected pair table. When the latest Kagi point is current, follow the spread direction if h_vol>2H; otherwise take the opposite pair direction.
- Exit conditions: Positions are forward-filled until a later Kagi signal changes them; no separate fixed stop or target is specified.
- Trade management: Long/short pair legs are held until the next Kagi signal; portfolio weights and costs are not specified.
- Target: Kagi reversal/signal change.
- Risk factors and risk management: Kagi threshold H selection and spread-volatility calculation are not fully specified in the strategy cells; no cost model is shown.
- Best known candlestick timeframe(s): Daily close histories are used.
- reported_metrics: none reported
- notes: The notebook compares 20-pair portfolio variants; only the signal logic is extracted.
- sources: [financialnoob/pairs_trading — Kagi direction and h_vol>2H trend/contrarian switch; code cells 47–48](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/18.pairs_trading_with_kagi.ipynb)

### G051 | Kalman Hedge-Ratio Cointegration Pair Reversion
- priority: Low — Dynamic hedge ratios are a useful twist, but parameters are optimized in-sample and cost assumptions are optional.
- direction: market-neutral | market: Equities | timeframe: Daily bars with a 200-session Kalman estimation window (daily-native)
- indicators: Kalman-filtered intercept and hedge ratio, residual-spread standard deviation
- Overview: A Kalman filter updates the cointegration hedge ratio and spread volatility for a Backtrader pair strategy. The filter's dynamic coefficients distinguish it from the static OLS z-score baseline.
- Stock or ETF Criteria: Pairs selected from cointegration/price-distance candidates; positive hedge ratio required.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Estimate Kalman state over the configured 200-day window; enter when the log residual exceeds the configured entry threshold (default 2 SD) and beta is positive, taking the opposite spread side.
- Exit conditions: Exit when the spread reaches the configured inner threshold (default 0.5 SD) or when the trade loss limit is hit.
- Trade management: Backtrader two-leg positions with borrow cost and commission options; position exposure is tied to hedge ratio.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Kalman estimation and thresholds are grid-searched on historical data; transaction-cost settings are optional.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: This source's Kalman filter estimates both intercept and hedge ratio; it is not the RL approach below.
- sources: [wywongbd/pairstrade-fyp-2019 — Kalman filter setup, hedge/spread updates, and trade logic](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/ptstrategy_cointegration_kalman.py#L65-L187); [wywongbd/pairstrade-fyp-2019 — 200-session Kalman estimation and entry/exit parameters](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/backtest_pair.py#L48-L70); [wywongbd/pairstrade-fyp-2019 — Kalman pair parameter search and backtest](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/jupyter_py/5d-Cointegration-Kalman-Grid-Search.ipynb)

### G052 | Kalman-Hedge-Ratio Pair Mean Reversion with Beta-Stability Gate
- priority: Low — This is a reproducible dynamic-hedge-ratio twist, but it is minute-frequency and not evaluated on daily bars.
- direction: market-neutral | market: US equities | timeframe: 1-minute bars (intraday-only)
- indicators: Dynamic Kalman alpha/beta, 120-bar EWMA spread z-score
- Overview: A Kalman filter adapts the pair hedge ratio and constructs a synthetic spread; a rolling EWMA z-score trades large dislocations only while the hedge ratio remains stable. The pair-selection notebook searches within categories and uses training data for the cointegration test.
- Stock or ETF Criteria: The notebook selects one low-p-value pair per asset category from its training sample; sample pair is selected from US equity data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Select the best training-period cointegrated pair in each category (minimum p-value, p<0.05). Compute z=(spread−EWMA120 mean)/(EWMA120 std), clipped to [−5,5]. Enter when |z|>2.5 and the current Kalman beta differs from the prior beta by <0.05; long when z<−2.5, short when z>2.5.
- Exit conditions: Close a long when z>−0.5 or a short when z<0.5; also close if beta drift exceeds 0.15.
- Trade management: Position is long/short spread with dynamic beta; transaction cost is 0.0005 per position change. Stop on beta drift >3×delta_min.
- Target: Spread mean reversion; entry at ±2.5 z and exit toward ±0.5 z.
- Risk factors and risk management: Minute data and a single selected pair limit generalization; beta stability and z-score estimates are sensitive to the training window.
- Best known candlestick timeframe(s): Minute bars only in the reported strategy; not applicable to daily-only data without recalibration.
- reported_metrics: none reported
- notes: The repo's separate Avellaneda–Stoikov quote engine in notebooks 03–04 is market-making/HFT and is excluded from this batch's strategy records.
- sources: [DuarteAbracos/statistical-arbitrage-kalman — Training-only cointegration pair selection, Kalman beta, and EWMA120 z-score; code cells 1, 5, and 6](https://github.com/DuarteAbracos/statistical-arbitrage-kalman/blob/d3f2a17bdf1a28262a9618139603a807a46c989d/notebooks/01_kalman_pairs.ipynb); [DuarteAbracos/statistical-arbitrage-kalman — Threshold=2.5, exit_threshold=0.5, delta_min=0.05, and 0.0005 position-change cost; code cells 1–3](https://github.com/DuarteAbracos/statistical-arbitrage-kalman/blob/d3f2a17bdf1a28262a9618139603a807a46c989d/notebooks/02_strategy_baseline.ipynb)

### G053 | LMAR Hidden-Regime Pair Spread Forecast
- priority: Low — The probabilistic latent-regime pair method is novel and reproducible, but selection risk and missing costs limit priority.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: OLS hedge spread, EM-fitted LMAR parameters, logistic mean-reversion probability
- Overview: An EM-fitted latent-regime model estimates the probability that a hedged pair spread is in a mean-reverting state. The strategy trades only when this probability is above a pair-specific historical 95th percentile.
- Stock or ETF Criteria: Equity pairs with positive OLS hedge beta and repeated converged EM estimates.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Select non-overlapping pairs with at least five converged EM fits and the notebook's parameter-significance screens. For each date compute p=sigmoid(delta0+delta1·|spread(t−1)|+delta2·|spread(t−2)|); enter opposite the spread when p>pq95, only during the first 25 test sessions.
- Exit conditions: Close when p<pq50 or after 50 sessions from the pair's first opening.
- Trade management: Pair positions use OLS hedge weights; the notebook evaluates up to ten pairs.
- Target: Mean reversion while the fitted regime probability remains high.
- Risk factors and risk management: The strategy selects pairs and probability cutoffs on a limited training sample; transaction costs are not specified.
- Best known candlestick timeframe(s): Daily data are native.
- reported_metrics: none reported
- notes: The source's test loop uses `i<25` for entries and closes on probability below pq50 or after 50 sessions.
- sources: [financialnoob/pairs_trading — EM parameter screens, pq95/pq50 probabilities, 25-session entry window and 50-session time stop; code cells 28–38](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/19.pairs_trading_with_lmar/19.pairs_trading_with_lmar.ipynb)

### G054 | Lead–Lag Follower Versus Market Strategy
- priority: Low — The signal is a plausible delayed-response thesis but the main portfolio parameters and cost treatment are unspecified.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Rolling pairwise lead–lag metrics C1/C2 or Lévy area; market return
- Overview: The strategy ranks equities as leaders or followers from rolling lead–lag dependence and trades followers against the market in the direction implied by the leaders' recent average return. It is a cross-sectional delayed-response effect rather than a stationary pair spread.
- Stock or ETF Criteria: Equities with available daily return histories; market return series is explicitly removed from the leader/follower matrix and used as hedge.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Over a trailing window w, compute C1/C2 or Lévy-area lead–lag matrices, select the top `num_leaders` and bottom `num_followers`; go long followers and short the market when mean leader return is nonnegative, otherwise reverse.
- Exit conditions: Recompute positions on each daily date; no fixed price target or stop is specified.
- Trade management: Long and short gross exposures are separately normalized to sum to one.
- Target: Rebalance as rolling leader/follower classification changes.
- Risk factors and risk management: Lookback w, leader/follower counts, costs, and execution details are not specified in the cited cells.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The notebook explicitly illustrates cases where the C1 metric misclassifies leaders; test the metric before use.
- sources: [financialnoob/pairs_trading — Rolling lead/follower construction and market-hedged positions using C1, C2, and Levy area; code cells 58–78](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/23.lead_lag_stat_arb/23.lead_lag_stat_arb.ipynb)

### G055 | PCA-Factor Residual Stat-Arb Course Strategies (R, NYU Jerzy FRE6871)
- priority: Low — Educational course code; strategies are canonical stat arb already covered by S071-style records.
- direction: long/short | market: US equities | timeframe: 1d (daily-native)
- indicators: PCA on returns, OU/AR(1) calibration, cointegration tests
- Overview: NYU FRE6871 course code implementing momentum and pairs/statistical-arbitrage backtests in R, including PCA-based factor residual returns (top PCs explaining >=80% variance) and standard cointegrated-pair mean reversion. Educational implementations of canonical stat-arb.
- Stock or ETF Criteria: not specified in source. Suggested: liquid US large/mid caps for the pair basket; same sector/industry for cointegration candidates.
- Stocks to avoid and known failure conditions: not specified in source. Suggested: avoid pairs failing ADF stationarity of the spread; PCA residual strategies degrade when factor loadings are unstable across the estimation window.
- Entry conditions: not specified in source. Spread z-score entries per Chan-style stat arb; exact thresholds embedded in lecture scripts. Suggested: z>2 short spread, z<-2 long.
- Exit conditions: not specified in source. Suggested: exit at |z|<0.5 or half-life-based time stop.
- Trade management: not specified in source. Suggested: dollar-neutral legs.
- Target: not specified in source. Suggested: spread reversion to 0.5-1 sigma.
- Risk factors and risk management: not specified in source. Suggested: cap pair count, volatility-scale positions.
- Best known candlestick timeframe(s): Daily bars; course runs on daily closes.
- reported_metrics: none reported
- external_refs: Prof. Jerzy Pawlowski, NYU FRE6871 Algorithmic Portfolio Management
- notes: Reviewed lecture R files; distinct executable rules are thin - treat as methodology reference.
- sources: [BessieChen/R-in-Finance — course description](https://github.com/BessieChen/R-in-Finance/blob/c0d0b26a580cd5183c869e8973fa9ef20599f3a3/README.md); [BessieChen/R-in-Finance — lecture code (PCA/pairs exercises)](https://github.com/BessieChen/R-in-Finance/blob/c0d0b26a580cd5183c869e8973fa9ef20599f3a3/Lecture_5%20.R)

### G056 | Quasi-Multivariate Correlated-Portfolio Spread Reversal
- priority: Low — The multi-leg residual is a distinct basket rule, but thresholds and execution costs are incompletely documented.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Rolling normalized prices, cross-sectional correlations, OLS basket weights
- Overview: For each anchor stock, the notebook forms a small correlated peer basket and trades the anchor's residual against that basket. It expands simple pairs trading to several hedge legs.
- Stock or ETF Criteria: Equity universe with each asset used at most once in the selected pair/basket set.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Using a trailing 494-session window, select peer stocks by correlation and compute either OLS or equal weights; enter short when the normalized residual exceeds +d and long when it falls below −d. The numerical d threshold is not specified in source.
- Exit conditions: Set the position to flat when the residual returns inside [−d,+d]; rebalance the peer basket every 10 sessions in the equal-weight version.
- Trade management: Equal-weight or OLS basket; an optional variant limits active positions to three.
- Target: Residual convergence to the basket mean.
- Risk factors and risk management: The notebook selects peers and weights from long historical windows; d and transaction-cost treatment are not specified.
- Best known candlestick timeframe(s): Daily bars are native.
- parameter_variants: OLS peer weights; equal-weight peers with 10-session basket updates; at most three simultaneous positions.
- reported_metrics: none reported
- notes: The source's equal-weight calculations use normalized cumulative prices; practical hedge-ratio and capital constraints require validation.
- sources: [financialnoob/pairs_trading — Trailing basket construction, OLS/equal-weight residual, and ±d positions; code cells 14–15 and 30–34](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/13.quasi_multivariate_pairs_trading.ipynb)

### G057 | Regime- and Momentum-Filtered Adaptive Z-Score Pair Strategy
- priority: Low — The rule set is unusually complete and costs are modeled, but its dynamic/regime complexity and lack of independent OOS evidence reduce priority.
- direction: market-neutral | market: US ETFs; default SPY/QQQ | timeframe: 1d (daily-native)
- indicators: 30-session spread z-score, volatility-scaled thresholds, 50/200 spread SMA regime, 5-day spread ROC
- Overview: This system combines a rolling spread z-score with volatility-adaptive thresholds, a spread trend/volatility regime filter, and short-term momentum checks. It avoids treating every extreme spread as a mean-reversion opportunity.
- Stock or ETF Criteria: Defaults to SPY/QQQ; configuration supports other cointegrated stocks, ETFs, futures, commodities, forex or crypto pairs.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Default config uses a 30-session z-score window, 252-session long-term lookback, 50/200-session regime SMAs, and dynamic entry thresholds initialized at ±2 then multiplied by a 100-session volatility ratio and clipped to 1.5–3.5σ. Long entries require z below the negative threshold, mean-reverting/down regime, and long-momentum filter; shorts require the symmetric conditions.
- Exit conditions: Exit when |z|<0.5, when a long z rises above +0.5, when a short z falls below −0.5, or when regime becomes Volatile_Trending. Pyramiding can add up to two levels at 0.5σ additional excursion.
- Trade management: Optional dynamic thresholds, up to two pyramid levels, 2.5×ATR stop, 4×ATR target, 2% risk/trade, 30% maximum position, 20% portfolio drawdown stop, 0.05% transaction cost and 0.02% slippage.
- Target: Z-score mean reversion; secondary exit on volatile-trending regime.
- Risk factors and risk management: Multiple filters and thresholds create overfit risk; default lookback/regime values are not validated out of sample.
- Best known candlestick timeframe(s): Daily bars are native; intraday can be used only after recalibrating the rolling windows.
- reported_metrics: none reported
- notes: Regime detection calls a spread trending when fast SMA exceeds slow SMA by 2%; high-volatility ratio >1.2 marks Volatile_Trending.
- sources: [XanderRobbins/Universal-Pairs-Trading-System — Rolling z-score, regime/momentum filters, dynamic thresholds, entries and exits](https://github.com/XanderRobbins/Universal-Pairs-Trading-System/blob/8b4a5952ded3aa6634481a3e45ef17a8cbc7aca4/strategy.py#L70-L290); [XanderRobbins/Universal-Pairs-Trading-System — Default pair, 30/252 lookbacks, ±2/0.5 thresholds, risk and costs](https://github.com/XanderRobbins/Universal-Pairs-Trading-System/blob/8b4a5952ded3aa6634481a3e45ef17a8cbc7aca4/config.py#L11-L92); [XanderRobbins/Universal-Pairs-Trading-System — Transaction costs, slippage and drawdown breaker](https://github.com/XanderRobbins/Universal-Pairs-Trading-System/blob/8b4a5952ded3aa6634481a3e45ef17a8cbc7aca4/backtester.py#L62-L116)

### G058 | Rolling Cointegration Pair Z-Score Prototype
- priority: Low — 
- overlap: S071 (existing catalog) — The code attempts rolling cointegration gating and residual z-score thresholds but computes each band as the current z-score ± the raw residual standard deviation, making the comparison self-referential and likely preventing signals.
- direction:  | market: Equities | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [je-suis-tm/quant-trading — Rolling 250-session cointegration, residual z-score, and pair-leg signal construction](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Pair%20trading%20backtest.py#L108-L183)

### G059 | Shumway–Stoffer State-Space Pair Mean Reversion
- priority: Low — The state-space model is a distinct, code-implemented pair-selection approach, but entry/exit details and out-of-sample evidence are incomplete.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Pair price ratio/spread, state-space Kalman filter, mean and variance regimes
- Overview: This method estimates latent spread dynamics with a Shumway–Stoffer/Kalman state-space model, screens for stable mean-reverting pairs, and forms long/short pair positions from deviations around estimated regime means. It differs from a fixed rolling z-score through the fitted dynamic latent state.
- Stock or ETF Criteria: Pairs of equities with at least 30 formation observations and stable state-space parameters.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Formation uses a rolling 30-session window and fits the state-space model with 150 EM iterations. Retain pairs with 0<B<1 and C<D; the later strategy uses the fitted spread/regime parameters to assign pair direction.
- Exit conditions: Pairs are closed when the state-dependent spread reverts through the relevant estimated mean; exact threshold and exit constants are not specified in source.
- Trade management: Notebook approaches allocate equal capital to the selected long and short pair sets; no transaction costs are specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The pair screen and state parameters are estimated repeatedly on short windows; the model's strategy exit thresholds are not completely documented in source.
- Best known candlestick timeframe(s): Daily price histories are used; the state-space method can be applied to other bar sizes with recalibration.
- reported_metrics: none reported
- notes: The notebook includes multiple strategy variants; exact rules beyond the stated model screens are left unspecified rather than inferred.
- sources: [financialnoob/pairs_trading — Shumway–Stoffer smoother, rolling pair selection, and strategy variants; code cells 1, 6, 8, 11, 14, and 17](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/10.pairs_trading_GMC.ipynb)

### G060 | Soybean Crush-Spread Fair-Value Mean Reversion
- priority: Low — The processing-margin thesis and variants are distinct, but the private signal implementation and unstable fair-value model lower confidence.
- direction: long/short | market: Soybean, soybean-meal, and soybean-oil futures | timeframe: Daily futures (daily-native)
- indicators: Soybean crush spread, 5-session spread moving average, optional regression fair value
- Overview: The strategy trades soybean-product processing margins as a mean-reverting spread and compares a moving-average trigger with an in-sample regression fair-value filter. It also contrasts long-winter with short-growing-season exposure.
- Stock or ETF Criteria: Soybean, soybean-meal, and soybean-oil futures; contract multipliers are applied in the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: The `CrushSpreadSimon1999InSample` variants use `ma_lookback=5`, `ma_entry=100` or 300, `ma_exit=100`, optional fair-value filter with entry 0 or 0.1, and a fair-value fit ending 1995-02-28; long-only and short-only variants are tested.
- Exit conditions: The private signal class receives `ma_exit` and fair-value settings, but the exact exit computation is not in the clone.
- Trade management: Long-only and short-only variants; sizing and transaction costs are delegated to private `vivace`.
- Target: Mean reversion toward the spread's moving average or model fair value.
- Risk factors and risk management: Fair-value parameters are fit through 1995 and the notebook notes residual drift out of sample; net costs are unavailable.
- Best known candlestick timeframe(s): Daily futures bars; seasonal variants use calendar months.
- parameter_variants: Long/short; with/without fair value; MA entry 100/300; fair-value entry 0/0.1; calibration end 1995-02-28.
- reported_metrics: none reported
- external_refs: Simon (1999), as named in the notebook title; complete publication details are not specified in the inspected cells.
- notes: Notebook says the fair-value residual drifts out of sample and the seasonal strategy is not attractive.
- sources: [rolling-panda-san/notebooks — Crush-spread leg definitions, MA/fair-value variants and seasonality; code cells 73–109 and 117–124](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_crush_spread_stat_arb.ipynb)

### G061 | Tick-Level Beta-Expected-Price Pair Reversion
- priority: Low — The pair logic is explicit but is high-frequency, lacks risk/transaction-cost modeling, and has a possible signal-direction defect.
- direction: long/short | market: Broker-supplied pair; examples include FX | timeframe: Tick data with a 30-second resampled statistic and 1-hour history window (intraday-only)
- indicators: Mean price ratio beta, relative volatility ratio, tick midpoint
- Overview: The model pairs two instruments using beta=mean(price A)/mean(price B), then trades deviations of A from beta×B conditioned on a volatility-ratio filter. The source describes this as mean reversion, although its variable names and implemented conditions appear directionally inconsistent.
- Stock or ETF Criteria: Two broker-traded instruments with tick-level midpoint data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Recompute beta and relative volatility from the latest one-hour midpoint history; if volatility ratio>1 and A>beta×B, set buy signal; if ratio<1 and A<beta×B, set sell signal.
- Exit conditions: Close a long on a sell signal and a short on a buy signal; no stop, target, or maximum holding time is specified.
- Trade management: Fixed `trade_qty` per spread order; buy one leg and sell the other; market orders are submitted to Interactive Brokers.
- Target: Reverse/close on opposite signal.
- Risk factors and risk management: No trading costs or stop controls are specified; variable names `is_overbought`/`is_oversold` conflict with their price-vs-expected-price comparisons.
- Best known candlestick timeframe(s): Tick/high-frequency only; not applicable to daily bars.
- reported_metrics: none reported
- notes: Although the project is categorized as mean reversion, its docstring calls this an HFT model and its code's BUY condition purchases A when A is above its beta-implied value.
- sources: [jamesmawm/High-Frequency-Trading-Model-with-IB — Beta/volatility calculations, signal direction, and pair order lifecycle](https://github.com/jamesmawm/High-Frequency-Trading-Model-with-IB/blob/8e96ade54a8e3eb94ead04827a846225ae122864/models/hft_model_1.py#L79-L214)

### G062 | Tick-Level Kalman Cointegration Error Reversion
- priority: Low — The adaptive pair residual is explicitly implemented, but it is high-frequency with no cost-aware validation.
- direction: market-neutral | market: User-configured equity/FX pair | timeframe: Tick updates with daily historical initialization (intraday-only)
- indicators: Recursive Kalman hedge ratio and last-15-error standard deviation
- Overview: The strategy updates a pair hedge ratio online and opens a two-leg position when the current residual exceeds the standard deviation of recent errors. It closes when the residual crosses zero.
- Stock or ETF Criteria: Pair and contracts are supplied by the user; historical bootstrap uses daily bars.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: After more than 30 error observations, compute SD from the latest 15 errors; if |error|>SD, go long the pair when error<0 or short when error>0. The Kalman state is updated for every new price observation.
- Exit conditions: Close both legs when the residual crosses zero against the held direction.
- Trade management: Leg quantities use the live hedge ratio and a leverage cap of 4; broker market/limit order handling is configurable.
- Target: Spread residual crosses zero.
- Risk factors and risk management: Tick-frequency orders, no explicit stop, and the leverage cap create material execution and short-leg risk.
- Best known candlestick timeframe(s): Tick/market-data updates; not a daily-only implementation.
- reported_metrics: none reported
- notes: The project is a backtesting framework and the pair universe is caller-specified.
- sources: [lukstei/trading-backtest — Online hedge update, one-SD entry and zero-cross exit](https://github.com/lukstei/trading-backtest/blob/f9598b23d0ffaf18fdab3472cb3ed70cbf2b5c86/src/main/java/org/lst/trading/main/strategy/kalman/CointegrationTradingStrategy.java#L44-L97); [lukstei/trading-backtest — Recursive cointegration/Kalman state update](https://github.com/lukstei/trading-backtest/blob/f9598b23d0ffaf18fdab3472cb3ed70cbf2b5c86/src/main/java/org/lst/trading/main/strategy/kalman/Cointegration.java)

### G063 | Treasury Futures Z/T-Score Relative-Value Entry
- priority: Low — The code has an explicit multi-factor entry and roll-aware exit but lacks a fixed configuration and validated results.
- direction: market-neutral | market: Treasury futures | timeframe: Daily (daily-native)
- indicators: Portfolio spread z-score and T-score, contract duration/roll schedule
- Overview: The backtester forms weighted Treasury-futures portfolios and enters when both a spread z-score and T-score breach configured lower thresholds. It exits on profit/loss bands or before contract roll.
- Stock or ETF Criteria: Treasury futures with duration/margin/point-value inputs and a roll calendar.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Enter when ZScore≤−triggerS and TScore≤−triggerT, provided the date precedes the roll date and the portfolio is below its maximum concurrent-position count; trigger levels are user-configured.
- Exit conditions: Close when portfolio value changes exceed the configured upper/lower tick-size exit levels or at the roll date.
- Trade management: Maximum concurrent positions are capped; margin, point value, transaction-cost coefficient and position weights are inputs.
- Target: Configured portfolio-value movement measured in tick-size units.
- Risk factors and risk management: Trigger/exit parameters and contract universe are not specified by the backtest source; roll handling is essential.
- Best known candlestick timeframe(s): Daily futures bars.
- reported_metrics: none reported
- notes: The file contains a probable setter bug (`set_exitDownLevel` does not assign the value); verify before implementation.
- sources: [jerryxyx/TreasuryFutureTrading — Portfolio z/T-score calculation, entry gate, exit thresholds and roll handling](https://github.com/jerryxyx/TreasuryFutureTrading/blob/ce236a94e74f561f01026940e96a85765d2cbd06/BackTesting.py#L204-L274)

### G064 | Two-Regime Markov-Switching Pair Spread
- priority: Low — Regime-aware mean reversion is a distinct rule, but its operative probability and threshold parameters are not specified.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Pair price ratio, two-regime Markov regression, regime-specific means/volatilities
- Overview: A two-state Markov model estimates distinct spread means and variances, and trades deviations from the active regime's mean only when that regime is sufficiently probable. This can adapt pair entries to changing spread behavior.
- Stock or ETF Criteria: Pairs selected from the equity universe with training-period cointegration p-value below 0.10.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Formation ranks pairs by Kendall tau, keeps non-overlapping pairs with Engle–Granger p<0.10, then fits a two-regime switching-variance model on trailing 250-session windows. With delta=1.645 and rho=0.7, short above a regime mean + delta·sigma and long below mean − delta·sigma; the high-regime short and low-regime long are unconditional, while high-regime long and low-regime short require the corresponding regime probability > rho.
- Exit conditions: The notebook recomputes a fresh position each day from the four regime/threshold conditions; when none holds, the pair is flat.
- Trade management: Equal-weight selected pairs, with each long/short leg sized symmetrically.
- Target: Mean reversion to the regime-specific spread mean.
- Risk factors and risk management: Repeatedly fitting a switching model on a 250-session rolling window can be unstable; no costs are specified.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: Notebook reuses the full tested history for regime fitting in some exploratory cells; only training/trading splits should be used for a valid backtest.
- sources: [financialnoob/pairs_trading — Cointegration/association screen, rolling Markov regimes, and regime-conditioned positions; code cells 9, 20, and 25](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/12.pairs_trading_markov_switching.ipynb)

### G065 | Wavelet-Filtered Cointegrated Pair Reversion
- priority: Low — Denoising is a distinct signal-processing twist, but exact preprocessing parameters are not sufficiently pinned for replication.
- direction: market-neutral | market: Equities | timeframe: 1d (daily-native)
- indicators: Wavelet-denoised prices/returns, cointegration, OLS residual spread
- Overview: A wavelet preprocessing step is applied before pair screening and spread estimation, aiming to reduce high-frequency noise in daily prices. The residual still uses a thresholded mean-reversion position rule.
- Stock or ETF Criteria: Pairs passing the notebook's cointegration tests; exact universe and wavelet parameter are not specified.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Filter prices/returns with the notebook's wavelet transform, screen candidate pairs for stationarity/cointegration, and enter short above +2 spread standard deviations or long below −2; exact filter level and selected wavelet are not specified in source.
- Exit conditions: Exit when the spread crosses zero; code also evaluates a top-ten pair selection.
- Trade management: Equal-weight selected pairs; the notebook limits one variant to ten non-overlapping pairs.
- Target: Spread reversion to zero.
- Risk factors and risk management: Wavelet filter details and transaction costs are not specified in source; in-sample ranking may bias pair selection.
- Best known candlestick timeframe(s): Daily data are used.
- reported_metrics: none reported
- notes: The notebook does not fully document the selected wavelet and level in the trading cells.
- sources: [financialnoob/pairs_trading — Wavelet filtering and pair spread signals; code cells identified under the notebook's trading section](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/21.pairs_trading_with_wavelet_transform.ipynb); [financialnoob/pairs_trading — Wavelet transform variant of the Granger-filtered pair screen; code cells 123–146](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/14.pairs_trading_granger_causality.ipynb)

### G066 | Zipline Cointegration Pair Z-Score (z_signal=2, z_exit=0, lookback=200)
- priority: Low — Exact S071 mechanics; included for parameter confirmation (z=2/0, 200 lookback).
- overlap: S071 (existing catalog) — z_signal=2, z_exit=0, lookback=200 - standard params; ARIMA notebook is analysis-only.
- direction: market-neutral | market: US equities | timeframe: 1d (daily-native)
- indicators: Spread z-score on 200-observation lookback; OLS hedge ratio
- Overview: Zipline implementation of the textbook cointegrated-pair z-score strategy: enter at |z|>2, exit at z=0, hedge ratio from 200-day regression. Also includes an ARIMA modeling notebook (no distinct strategy).
- reported_metrics: none reported
- sources: [grananqvist/Machine-Learning-Trading-Strategies — z_signal/z_exit/lookback params](https://github.com/grananqvist/Machine-Learning-Trading-Strategies/blob/4c9f16f9a788b86d8a393cf0533dd8ce1ce17b20/3%20-%20Cointegration%20Mean%20Reversion%20Pairs%20Trading.ipynb); [grananqvist/Machine-Learning-Trading-Strategies — ARIMA analysis](https://github.com/grananqvist/Machine-Learning-Trading-Strategies/blob/4c9f16f9a788b86d8a393cf0533dd8ce1ce17b20/2%20-%20ARIMA%20time-series%20modelling.ipynb)


## Trend Following and Breakout

### G067 | Mount Lucas (MLM) Futures Trend Index - 200d MA + Universe Vol Gate, Monthly Execution
- priority: High — Published benchmark CTA construction with explicit contract mapping and a vol regime gate - directly portable to ETF futures proxies on daily bars.
- direction: long/short | market: futures (grains, cattle, energy, metals, financials via IBKR ContFuture) | timeframe: 1d signals, monthly execution (daily-native)
- indicators: 200-day SMA per contract; 20-day realized vol averaged across the universe
- Overview: Automated Mount Lucas Management-style trend index: signal = +1/-1 per contract based on price vs its 200-day MA on continuous futures data; orders are placed only on the 25th of each month AND only when the average 20-day realized volatility across the whole futures universe exceeds a threshold (vol regime gate). MLM is the canonical simple diversified CTA benchmark - long-run positive expectancy from trend persistence across commodities/financials.
- Stock or ETF Criteria: Diversified futures universe (ZC ZW ZS ZM ZL LE CL + metals/financials); ContFuture for signals, front-month for execution.
- Stocks to avoid and known failure conditions: Trend regimes can last years without a signal change - the monthly check is a convenience not a filter; vol gate can skip all signals in quiet regimes; futures roll handling is simplified.
- Entry conditions: Per contract: long if close > SMA200 else short; trade only on day==25 and only if mean(universe 20d realized vol) > threshold.
- Exit conditions: Signal flip at the next monthly check.
- Trade management: Equal signals across universe; contracts mapped from ContFuture to front month.
- Target: Long-run CTA trend premium.
- Risk factors and risk management: Universe-level vol gate; per-contract market orders; no per-position stops (index-style).
- Best known candlestick timeframe(s): Daily bars, monthly rebalance cadence.
- reported_metrics: none reported
- external_refs: Mount Lucas Management MLM Index (200-day MA benchmark CTA)
- notes: The monthly-execution + universe-vol-gate combination is the distinctive twist vs plain SMA200 trend.
- sources: [amstrdm/mlm-trend-following — universe, signal=close>MA200, 25th-of-month + avg-vol gate](https://github.com/amstrdm/mlm-trend-following/blob/9e76af882e313297224929b488ad79bc360c51b2/main.py)

### G068 | Stochastic-Oversold-Exit + RSI-50 Confirmation with Chandelier(22,2) Trailing Exit
- priority: High — Complete, daily-native, precise rule set combining oversold-exit timing + trend gate + vol trailing - immediately implementable.
- direction: long/short | market: equities (portfolio of tickers) | timeframe: 1d (daily-native)
- indicators: RSI(14) 50-level; Stochastic %K(14-ish); Chandelier Exit 22-period 2xATR trailing stop
- Overview: Long when Stochastic just LEAVES oversold (prev %K<20, now >25) AND RSI>=50 confirms trend is at least neutral-up; short mirror (prev %K>80 -> <75 with RSI<50). Positions held until the Chandelier trailing stop (22-bar high/low -/+ 2*ATR) is hit. A clean 'reversal-out-of-extreme inside trend' entry plus a proper volatility trailing exit.
- Stock or ETF Criteria: Any equity/ETF daily bars.
- Stocks to avoid and known failure conditions: Stochastic-20 re-entry signals can fire repeatedly in sustained downtrends (RSI>=50 helps but whipsaws); Chandelier gives back open profit by design.
- Entry conditions: Long: Stoch[t-1]<20 AND Stoch[t]>25 AND RSI[t]>=50. Short: Stoch[t-1]>80 AND Stoch[t]<75 AND RSI[t]<50.
- Exit conditions: Chandelier(22,2): long exits when close < HHV(22)-2*ATR; short mirror on LLV(22)+2*ATR.
- Trade management: One position per ticker, portfolio loop across tickers.
- Target: Trail to Chandelier (no fixed TP).
- Risk factors and risk management: ATR trailing stop is the risk manager; entries gated by RSI trend filter.
- Best known candlestick timeframe(s): Daily bars (native).
- reported_metrics: none reported
- notes: Entry = stochastic leaving oversold, not oversold itself - timing detail matters.
- sources: [amoghathalekar/Stochastic-RSI-Trend-Following-Strategy — signal loop + CHSTOP(22,2)](https://github.com/amoghathalekar/Stochastic-RSI-Trend-Following-Strategy/blob/4f4d2a40804829caba0577b7637c3665d6c0d6d6/RSI-Stochastics%20strategy%20NEW%20-%20github%20copy.py)

### G069 | 20-Day-High Breakout Trend (Hyperliquid)
- priority: Medium — Crossref to Donchian breakout family; crypto perp market is the only distinction.
- overlap: S036 (existing catalog) — 20-day-high breakout (Donchian S1-equivalent) on crypto perps; maxOpenPositions config.
- direction: long | market: crypto perps (Hyperliquid) | timeframe: 1d (daily-native)
- indicators: 20-day high
- Overview: Basic trend-following bot: enters when price breaks the 20-day high, manages multiple open positions per config. A Donchian-channel breakout - the crypto-native instance of S036.
- reported_metrics: none reported
- sources: [kalipsot/Hyperliquid-Basic-Trend — twenty_day_high breakout logic](https://github.com/kalipsot/Hyperliquid-Basic-Trend/blob/ef74603fc31fc183b995806cd252daf0839d7b81/main.py)

### G070 | ATR SuperTrend Direction-Reversal Strategy
- priority: Medium — The signal and reversal exits are reproducible and daily-adaptable, but the sources do not establish a robust out-of-sample edge after costs.
- direction: long/short | market: Instrument supplied by chart/backtest | timeframe: Daily bars in the VectorBT example; chart timeframe otherwise (daily-native)
- indicators: ATR-based SuperTrend bands and direction state
- Overview: The trailing ATR band defines the active trend; a close crossing the prior opposing band reverses direction. A new position is taken on each direction flip, allowing the position to remain exposed through persistent trends.
- Stock or ETF Criteria: Liquid instrument with OHLC bars; the source does not define a specific asset universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For the Pine variant, default ATR period is 10 and multiplier is 3: go long when trend changes from −1 to +1 and short when it changes from +1 to −1. The VectorBT variant defaults to period 7 and multiplier 3 and enters when the corresponding SuperTrend long/short line first becomes non-null.
- Exit conditions: Exit on the opposite SuperTrend direction. The Pine script's optional 15% stop is disabled by default; the VectorBT example charges 0.1% fees.
- Trade management: Pine default quantity is 5 units; the VectorBT notebook uses signal-based positions and models 0.1% fees.
- Target: No fixed profit target; trend reversal is the exit.
- Risk factors and risk management: Whipsaws in range-bound markets; the Pine strategy's optional percentage stop is off by default.
- Best known candlestick timeframe(s): Daily is supported by the VectorBT example; higher-frequency chart data may be used by the Pine version.
- parameter_variants: Pine period=10/multiplier=3; VectorBT period=7/multiplier=3, with a parameter grid explored in the notebook.
- reported_metrics: none reported
- notes: The VectorBT notebook shifts the generated signals forward before portfolio execution.
- sources: [rahulmr/Pinescript-SuperTrend-Trading-Strategy — ATR SuperTrend direction flips, optional stop, and long/short order logic](https://github.com/rahulmr/Pinescript-SuperTrend-Trading-Strategy/blob/dfe2292e58583690271ddf515a991d84faa70563/supertrend.pinescript#L19-L128); [PacktPublishing/Python-for-Algorithmic-Trading-Cookbook — SuperTrend bands, direction changes, entries/exits, and 1 bp commission; code cells 11–35](https://github.com/PacktPublishing/Python-for-Algorithmic-Trading-Cookbook/blob/b201ffcb44c4a88c2604b94d41e3b2d5ca7c9ba2/06.%20Vector-Based%20Backtesting%20with%20VectorBT/06c_optimizing_the_supertrend_strategy_with_vectorbt_pro.ipynb)

### G071 | Keltner Mid-Channel Pullback-in-Trend EA
- priority: Medium — Precise pullback-in-trend spec with sensible filters; the bar-midpoint condition and parallel-signal ranging filter are distinctive.
- direction: long/short | market: FX (MT4) | timeframe: chart TF (param) (intraday-adaptable)
- indicators: Keltner channel (MA + ATR height), MA slope, bar midpoint vs MA, ATR for exits
- Overview: Buys when the MA is sloping UP, the bar's price midpoint is below the MA (pullback), and price is below the MA - i.e., a dip inside an uptrend bounded by the ATR channel; sells the mirror (MA sloping down, price above midpoint and MA). Exits are set 1 ATR from entry. A disciplined pullback-in-trend entry with an unusual 'midpoint of the bar' condition.
- Stock or ETF Criteria: FX pairs; requires uniform-volatility ranging markets for the optional parallel-signal filter.
- Stocks to avoid and known failure conditions: Pure MA-slope filter whipsaws in transitions; 1-ATR exits cut winners short in strong trends.
- Entry conditions: Buy: MA sloping up AND close (bar midpoint) below MA AND price below MA. Sell mirror. Optional 'parallel signal' second instance shifted by a multiple of MA period as a ranging-market filter; Tp/Sl spread-multiple filter rejects signals where SL-TP distance is too small vs spread.
- Exit conditions: TP/SL set 1 ATR from entry price.
- Trade management: PortfolioManager-based EA framework; per-signal parameters externalized.
- Target: 1 ATR (or per skew setting).
- Risk factors and risk management: ATR-boxed exits; spread filter avoids untradeable signals.
- Best known candlestick timeframe(s): Param TF; daily adaptation = 20-EMA pullback-in-uptrend with ATR stop (cf. S087).
- reported_metrics: none reported
- notes: Related to S087 (20-EMA pullback) - kept full for the Keltner/midpoint specifics.
- sources: [matthewkastor/Metatrader — entry/exit criteria](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/KeltnerPullbackTrader/README.md); [matthewkastor/Metatrader — EA](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/KeltnerPullbackTrader/KeltnerPullbackTrader.mq4)

### G072 | MBBS Three-Speed EWMA CTA Momentum
- priority: Medium — The model specifies multiple trend horizons and explicit portfolio risk scaling, but the source does not establish robust performance after costs.
- direction: long/short | market: Multi-asset dataset; exact asset list not specified in source | timeframe: Daily signals, approximately monthly rebalance (daily-native)
- indicators: Three short/long EWMA crossovers, response scaling, realized volatility
- Overview: The MBBS model combines three EWMA trend speeds and applies a nonlinear response to each trend signal. Volatility-parity/risk-parity weights and a 10% volatility target turn the directional signals into a diversified futures-style portfolio.
- Stock or ETF Criteria: Daily price and return histories for a diversified multi-asset universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each asset, compute EWMA crossover signals with default memory pairs [8,16,32] versus [24,48,96], mix them equally, and pass through the configured response scale (0.89) and response exponent (4).
- Exit conditions: Recompute the model signal at each monthly rebalance; no fixed price exit is specified.
- Trade management: Default equal mix of three speeds, volatility-parity weighting, 63-day price-volatility window, 252-day signal-volatility window, and 10% target volatility. Main.m also tests risk parity/equal weight and trend-quantity filters.
- Target: No fixed target; the portfolio is rebalanced monthly.
- Risk factors and risk management: The strategy depends on calibrated decay factors and asset-class return histories; no repo-claimed cost-aware OOS metrics are included here.
- Best known candlestick timeframe(s): Daily bars are native; monthly rebalance is appropriate for slower trend following.
- parameter_variants: EW, volParity, riskParity; noRule, overQuantity, and indQuantity signal filters.
- reported_metrics: none reported
- external_refs: Baz et al. 2015, as named in the modelMBBS.m source; full title is not specified there.
- sources: [blacksouane/QARMII_project — EWMA memory, response, weighting and trend-quantity options](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/MBBS/modelMBBS.m#L1-L64); [blacksouane/QARMII_project — Signal computation and allocation](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/MBBS/modelMBBS.m#L165-L240); [blacksouane/QARMII_project — EWMA crossover response function](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Signal/ewmaCO.m#L1-L80)

### G073 | Three-Day Pullback Above SMA200
- priority: Medium — 
- overlap: S007 (existing catalog) — Matches S007's 200-day trend gate and three-down-close entry; this notebook carries the signal for five sessions rather than exiting on a 5-day-MA reclaim.
- direction:  | market: US equities | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [rolling-panda-san/notebooks — Three consecutive down sessions and 200-session trend gate; code cell 4](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/equity_short_term_trading_connors.ipynb)

### G074 | Turtle Donchian Reversal (26/10)
- priority: Medium — 
- overlap: S036 (existing catalog) — Entry uses 26-bar high/low channels and exit uses 10 bars, rather than the catalog's 20/10 and 55/20 variants; 100% equity sizing and pyramiding=1 are used.
- direction:  | market: Chart instrument; not specified in source | timeframe: Chart timeframe (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [timchepeleff/turtles — Turtle channel entry/exit and 100% equity sizing](https://github.com/timchepeleff/turtles/blob/22cba516b257d836d119853c157fb7781ba8a139/pinescript/turtles.pine#L30-L46); [timchepeleff/turtles — Gekko implementation of the same Turtle channel family](https://github.com/timchepeleff/turtles/blob/22cba516b257d836d119853c157fb7781ba8a139/gekko/strategies/turtles.js)

### G075 | Adaptive Renko Trend Following with Partial De-Risking
- priority: Low — The rule is explicit and has fees/slippage but is a single-asset short-sample crypto system with aggressive parameter search.
- direction: long/short | market: Crypto spot, ETH/BTC on Bitfinex | timeframe: Daily close with 99-minute brick calibration (intraday-adaptable)
- indicators: Renko direction, optimized brick size, 96-bar absolute price differences
- Overview: The strategy holds in the current Renko direction and rebuilds the model when price reverses against the position. On every newly formed brick it partially reduces exposure, while an initial history-based optimization adapts the brick size to local price dispersion.
- Stock or ETF Criteria: ETH/BTC crypto pair; requires the Catalyst exchange history and Renko implementation.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: On model initialization, optimize brick size by maximizing Renko score over the interquartile range of absolute 96-bar differences from 877 bars of 99-minute history; build Renko history and target 100% exposure in the last brick direction.
- Exit conditions: When the latest Renko direction opposes the held position, flatten and rebuild the model. Otherwise, when a new brick forms, reduce the current position by 67%.
- Trade management: Leverage is 1.0; maker fee 0.1%, taker fee 0.2%, and slippage 0.05%. Parameter search tests n_history 150–1000, timeframe 10–100 minutes, diff_lag 1–100, and partial cover 0–1.
- Target: No fixed price target; trend reversal rebuilds the model.
- Risk factors and risk management: The code optimizes on a short 2018 period; the objective can select the best-performing fold when any fold is nonnegative, which risks selection bias.
- Best known candlestick timeframe(s): The example updates on daily bars using 99-minute history; a daily-only implementation would require recalibrating the Renko brick size.
- reported_metrics: none reported
- notes: The calibration horizon uses 99-minute bars and the backtest runs only 2018-11-01 through 2018-11-30.
- sources: [lamres/renko_trend_following_strategy_catalyst — Renko signal, sizing, costs, and reversal/partial-cover logic](https://github.com/lamres/renko_trend_following_strategy_catalyst/blob/d9f6f82585aa5fd66a5d5db3253985e8c9fca306/renko_trend_following.py#L19-L114); [lamres/renko_trend_following_strategy_catalyst — Five-fold parameter search and score objective](https://github.com/lamres/renko_trend_following_strategy_catalyst/blob/d9f6f82585aa5fd66a5d5db3253985e8c9fca306/renko_trend_following_optimizer.py#L12-L50)

### G076 | Awesome Oscillator Saucer and Zero-Line Momentum
- priority: Low — The rules are explicit but are a textbook indicator combination without realistic costs or evidence.
- direction: long/short | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: 5/34-session simple averages of median price, AO sign and three-bar saucer pattern
- Overview: The strategy uses the difference between 5- and 34-session simple moving averages of median price and adds a saucer-pattern trigger. It enters with oscillator direction and suppresses duplicate signals while a position is active.
- Stock or ETF Criteria: Equities downloaded from Yahoo Finance; no liquidity filter is specified.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Long when the 5/34 median-price oscillator is positive, or when the bearish-bar saucer conditions occur below zero: current bearish candle, prior two bullish candles, and rising AO while both latest oscillator readings remain negative. Short uses the mirrored bullish saucer above zero or negative oscillator direction.
- Exit conditions: Exit/reverse on the opposite signal; the portfolio code tracks cumulative position state.
- Trade management: Example portfolio uses $5,000 initial capital and 100 shares per position.
- Target: Reverse on opposite signal; no fixed profit target.
- Risk factors and risk management: The strategy has no costs, position-risk normalization, or out-of-sample validation in the cited code.
- Best known candlestick timeframe(s): Daily bars are used by the historical downloader.
- reported_metrics: none reported
- notes: The same file also runs a MACD moving-average benchmark; that separate rule is listed below.
- sources: [je-suis-tm/quant-trading — AO 5/34 calculation and saucer/zero-line signal generation](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Awesome%20Oscillator%20backtest.py#L55-L127); [je-suis-tm/quant-trading — Portfolio state and fixed-share backtest](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Awesome%20Oscillator%20backtest.py#L212-L235)

### G077 | CTA 1.0 Binary EWM Crossover
- priority: Low — The signal rules are explicit and daily-native, but the examples do not establish cost-aware out-of-sample performance.
- direction: long/short | market: Diversified futures/asset universe used by the notebook | timeframe: Daily bars (daily-native)
- indicators: EWM trend spread, robust volatility-adjusted returns, and (for Exp. 5) DCC correlations
- Overview: A basic time-series trend strategy maps each asset's fast-versus-slow EWM crossover to a signed position. The source notes that the binary position changes can create very high turnover.
- Stock or ETF Criteria: Multi-asset price series supplied to the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal is sign(EWM(price, com=32) − EWM(price, com=96)), with a 100-observation minimum warmup; position cash is signal × 5e6 in a 1e8 AUM portfolio.
- Exit conditions: Recompute the signed continuous target on each input bar; no discrete exit or maximum holding period is specified.
- Trade management: Fast/slow com 32/96; 5e6 cash-signal scale; AUM 1e8.
- Target: No fixed target; positions follow the continuous signal.
- Risk factors and risk management: The examples report strategy Sharpe but do not document transaction-cost assumptions in the cited cells; binary signals can have high turnover.
- Best known candlestick timeframe(s): Daily data are used by the notebooks.
- parameter_variants: Fast/slow com 32/96; 5e6 cash-signal scale; AUM 1e8.
- reported_metrics: none reported
- notes: Experiments 4–5 are portfolio-construction variants of the same robust oscillator; both are retained because their risk models differ materially.
- sources: [tschm/cs — Experiment 1: CTA 1.0 Binary EWM Crossover](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment1.py#L53-L72); [tschm/cs — Experiment 1: CTA 1.0 Binary EWM Crossover](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment1.py#L86-L89)

### G078 | CTA 2.0 Volatility-Scaled EWM Crossover
- priority: Low — The signal rules are explicit and daily-native, but the examples do not establish cost-aware out-of-sample performance.
- direction: long/short | market: Diversified futures/asset universe used by the notebook | timeframe: Daily bars (daily-native)
- indicators: EWM trend spread, robust volatility-adjusted returns, and (for Exp. 5) DCC correlations
- Overview: This variant scales the fast/slow EWM trend spread by estimated volatility, reducing the most extreme position changes of the binary prototype. It retains the same daily time-series trend thesis.
- Stock or ETF Criteria: Multi-asset price series supplied to the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute the 32/96 EWM spread with a 300-observation warmup, divide by an EWM volatility estimate (default 32), and use that signed signal as position cash.
- Exit conditions: Recompute the signed continuous target on each input bar; no discrete exit or maximum holding period is specified.
- Trade management: Fast/slow EWMs 32/96; volatility window 32; 300-row warmup; AUM 1e8.
- Target: No fixed target; positions follow the continuous signal.
- Risk factors and risk management: The examples report strategy Sharpe but do not document transaction-cost assumptions in the cited cells; binary signals can have high turnover.
- Best known candlestick timeframe(s): Daily data are used by the notebooks.
- parameter_variants: Fast/slow EWMs 32/96; volatility window 32; 300-row warmup; AUM 1e8.
- reported_metrics: none reported
- notes: Experiments 4–5 are portfolio-construction variants of the same robust oscillator; both are retained because their risk models differ materially.
- sources: [tschm/cs — Experiment 2: CTA 2.0 Volatility-Scaled EWM Crossover](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment2.py#L53-L76); [tschm/cs — Experiment 2: CTA 2.0 Volatility-Scaled EWM Crossover](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment2.py#L89-L94)

### G079 | CTA 3.0 Robust Volatility-Adjusted Oscillator
- priority: Low — The signal rules are explicit and daily-native, but the examples do not establish cost-aware out-of-sample performance.
- direction: long/short | market: Diversified futures/asset universe used by the notebook | timeframe: Daily bars (daily-native)
- indicators: EWM trend spread, robust volatility-adjusted returns, and (for Exp. 5) DCC correlations
- Overview: A robustly filtered, volatility-adjusted cumulative price series is converted to a scaled EWM-difference oscillator and passed through tanh. The design aims to make signal magnitudes comparable across assets while limiting outlier effects.
- Stock or ETF Criteria: Multi-asset price series supplied to the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Apply `vol_adj` to price with volatility window 32 and clip 3; cumulatively sum, compute the 32/96 scaled EWM-difference oscillator, divide by EWM volatility (default slow window), then apply tanh.
- Exit conditions: Recompute the signed continuous target on each input bar; no discrete exit or maximum holding period is specified.
- Trade management: Fast/slow 32/96; volatility 32; clip 3; 300-observation warmup.
- Target: No fixed target; positions follow the continuous signal.
- Risk factors and risk management: The examples report strategy Sharpe but do not document transaction-cost assumptions in the cited cells; binary signals can have high turnover.
- Best known candlestick timeframe(s): Daily data are used by the notebooks.
- parameter_variants: Fast/slow 32/96; volatility 32; clip 3; 300-observation warmup.
- reported_metrics: none reported
- notes: Experiments 4–5 are portfolio-construction variants of the same robust oscillator; both are retained because their risk models differ materially.
- sources: [tschm/cs — Experiment 3: CTA 3.0 Robust Volatility-Adjusted Oscillator](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment3.py#L77-L106); [tschm/cs — Experiment 3: CTA 3.0 Robust Volatility-Adjusted Oscillator](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment3.py#L122-L129); [tschm/TinyCTA — Analytic EWM-difference oscillator used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/osc.py#L40-L90); [tschm/TinyCTA — Robust volatility adjustment used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/util.py#L1-L120)

### G080 | CTA 4.0 Robust Oscillator with Risk-Parity Allocation
- priority: Low — The signal rules are explicit and daily-native, but the examples do not establish cost-aware out-of-sample performance.
- direction: long/short | market: Diversified futures/asset universe used by the notebook | timeframe: Daily bars (daily-native)
- indicators: EWM trend spread, robust volatility-adjusted returns, and (for Exp. 5) DCC correlations
- Overview: This system uses the robust CTA 3.0 trend oscillator but changes portfolio construction to risk-aware, volatility-scaled positions. It is an allocation variant of the same signal rather than a new entry signal.
- Stock or ETF Criteria: Multi-asset price series supplied to the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use the tanh of the 32/96 scaled EWM oscillator on volatility-adjusted cumulative prices (volatility 32, clip 4.2); allocate with the experiment's volatility/risk-parity position function.
- Exit conditions: Recompute the signed continuous target on each input bar; no discrete exit or maximum holding period is specified.
- Trade management: Fast/slow 32/96; volatility 32; clip 4.2; risk-parity allocation.
- Target: No fixed target; positions follow the continuous signal.
- Risk factors and risk management: The examples report strategy Sharpe but do not document transaction-cost assumptions in the cited cells; binary signals can have high turnover.
- Best known candlestick timeframe(s): Daily data are used by the notebooks.
- parameter_variants: Fast/slow 32/96; volatility 32; clip 4.2; risk-parity allocation.
- reported_metrics: none reported
- notes: Experiments 4–5 are portfolio-construction variants of the same robust oscillator; both are retained because their risk models differ materially.
- sources: [tschm/cs — Experiment 4: CTA 4.0 Robust Oscillator with Risk-Parity Allocation](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment4.py#L59-L97); [tschm/TinyCTA — Analytic EWM-difference oscillator used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/osc.py#L40-L90); [tschm/TinyCTA — Robust volatility adjustment used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/util.py#L1-L120)

### G081 | CTA 5.0 DCC Correlation-Aware Risk Parity
- priority: Low — The signal rules are explicit and daily-native, but the examples do not establish cost-aware out-of-sample performance.
- direction: long/short | market: Diversified futures/asset universe used by the notebook | timeframe: Daily bars (daily-native)
- indicators: EWM trend spread, robust volatility-adjusted returns, and (for Exp. 5) DCC correlations
- Overview: The fifth experiment keeps the same robust trend oscillator and sizes positions using an exponentially weighted dynamic conditional correlation estimate. It changes cross-asset risk allocation rather than signal direction.
- Stock or ETF Criteria: Multi-asset price series supplied to the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use the CTA 3.0 tanh oscillator (fast/slow 32/96, volatility 32, clip 4.2); estimate EWM covariance/correlation and compute daily risk-parity positions from the shrunk correlation matrix.
- Exit conditions: Recompute the signed continuous target on each input bar; no discrete exit or maximum holding period is specified.
- Trade management: Fast/slow 32/96; volatility 32; clip 4.2; EWM correlation and shrinkage set by notebook controls.
- Target: No fixed target; positions follow the continuous signal.
- Risk factors and risk management: The examples report strategy Sharpe but do not document transaction-cost assumptions in the cited cells; binary signals can have high turnover.
- Best known candlestick timeframe(s): Daily data are used by the notebooks.
- parameter_variants: Fast/slow 32/96; volatility 32; clip 4.2; EWM correlation and shrinkage set by notebook controls.
- reported_metrics: none reported
- notes: Experiments 4–5 are portfolio-construction variants of the same robust oscillator; both are retained because their risk models differ materially.
- sources: [tschm/cs — Experiment 5: CTA 5.0 DCC Correlation-Aware Risk Parity](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment5.py#L63-L65); [tschm/cs — Experiment 5: CTA 5.0 DCC Correlation-Aware Risk Parity](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment5.py#L81-L164); [tschm/cs — Experiment 5: CTA 5.0 DCC Correlation-Aware Risk Parity](https://github.com/tschm/cs/blob/7ca7b7709c6aee19b1edc4e6b7d6f6eb7f4d6c89/book/marimo/notebooks/Experiment5.py#L168-L187); [tschm/TinyCTA — Analytic EWM-difference oscillator used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/osc.py#L40-L90); [tschm/TinyCTA — Robust volatility adjustment used by the experiment](https://github.com/tschm/TinyCTA/blob/31cce37815384e161b069b3216a58aecec3a5542/src/tinycta/util.py#L1-L120)

### G082 | Dual Thrust Opening-Range Breakout
- priority: Low — The entry/exit window is explicit but the strategy is session-specific and execution-sensitive.
- direction: long/short | market: FX currency pairs | timeframe: Intraday, London session (intraday-only)
- indicators: Prior daily high/low/open/close range and London-session price
- Overview: A dual-thrust rule sets asymmetric upper/lower breakout thresholds from a prior-session range and trades the first London-session breakout. The code reverses when the opposite threshold is crossed and liquidates at the end of the London trading window.
- Stock or ETF Criteria: FX pairs with intraday bars and prior-session OHLC range.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: At 03:00 ET, set upper=open_reference + param×range and lower=open_reference − (1−param)×range; enter long above upper or short below lower, allowing one reversal if price crosses the opposite threshold.
- Exit conditions: Flatten all open exposure at 12:00 ET; no stop loss is used before the session close.
- Trade management: At most one directional signal at a time; positions are forcibly closed by 12:00 ET.
- Target: End-of-session liquidation; no intraday profit target.
- Risk factors and risk management: The threshold coefficient `param` is not fixed in source; gap/slippage and costs are not modeled.
- Best known candlestick timeframe(s): Intraday London-session data only; not applicable to daily bars.
- reported_metrics: none reported
- notes: The notebook comments describe an opening-range rule; exact data preprocessing and `param` choice are not pinned.
- sources: [je-suis-tm/quant-trading — Opening-range thresholds, breakout/reversal signal and noon liquidation](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Dual%20Thrust%20backtest.py#L94-L166)

### G083 | Golden-Cross Moving-Average Trend
- priority: Low — 
- overlap: S089 (existing catalog) — Uses the standard 50/200 golden cross and opposite cross to exit, but does not apply S089's rising-200-day-slope filter.
- direction:  | market: Equities; code example uses TSLA | timeframe: 1d (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [yungalyx/NoisyBoyAlgotrader — 50/200-day moving-average crossover entries/exits](https://github.com/yungalyx/NoisyBoyAlgotrader/blob/6c5783b0a8d602c1455e34e9d9eb664edc7b0b43/src/strategies/crossover.py#L11-L34)

### G084 | Leveraged ETF 2 PM Trend-Day Close Trade
- priority: Low — The entry and exit are reproducible and execution costs are modeled, but the edge is intraday-only and the notebook's threshold is selected in-sample.
- direction: long/short | market: US leveraged equity ETFs | timeframe: 1-minute bars; signal at 14:00 ET (intraday-only)
- indicators: Return from prior session close to the 14:00 bar open
- Overview: A large same-day move in leveraged ETFs is treated as a trend-day continuation signal. The model enters in the direction of the move during the afternoon and exits at the close, so it captures only the final intraday segment.
- Stock or ETF Criteria: Leveraged ETF universe defined by QuantRocket; eligible US stock listings.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: At 14:00, compute (14:00 open - prior 15:59 close)/prior close; buy if >6%, short if <−6%, otherwise stay flat. The backtest uses the 14:00 bar close as the 14:01 entry.
- Exit conditions: Exit at the 15:59 bar close with a market-on-close order.
- Trade management: Allocate 20% of capital per active position, capped at 100% gross exposure. Commission is $0.005/share and slippage is 3 bps.
- Target: Fixed same-day close; no profit target.
- Risk factors and risk management: Leveraged ETFs can gap and suffer severe intraday reversals; the strategy relies on a 2 PM signal and has no explicit stop.
- Best known candlestick timeframe(s): Only 1-minute/intraday data; not applicable to daily-only bars.
- reported_metrics: none reported
- notes: The notebook documents 14:00 signal and market-close execution; the intraday return cannot be faithfully mapped to daily bars.
- sources: [quantrocket-codeload/trend-day — Signal, 6% threshold, position sizing, commissions, slippage, and same-day exit](https://github.com/quantrocket-codeload/trend-day/blob/52d6884ed40241934cf45016f52cd55fa0131326/trend_day/trend_day.py#L19-L99); [quantrocket-codeload/trend-day — Parameter scan reports the 6% threshold as best; code cell 4](https://github.com/quantrocket-codeload/trend-day/blob/52d6884ed40241934cf45016f52cd55fa0131326/trend_day/Part3-Parameter-Scans.ipynb)

### G085 | Local-Extreme Futures Breakout
- priority: Low — A breakout approach is identified but the decisive lookback/trigger and cost model are unavailable.
- direction: long/short | market: Global futures | timeframe: Daily futures; month-end rebalance (daily-native)
- indicators: Private `BreakoutLocalExtreme` signal and volatility scaling
- Overview: The notebook compares a local-extreme breakout signal with time-series momentum across a diversified futures universe. A volatility scaler controls portfolio risk.
- Stock or ETF Criteria: Moskowitz 2012 global futures universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `BreakoutLocalExtreme()` and multiply by EWM volatility scaling (com=60, target 40%, cap 0.95), carried to month-end; breakout lookback and trigger price are not specified in the cloned notebook.
- Exit conditions: Rebalance at month-end; no separate exit threshold is specified.
- Trade management: Equal-weight instruments, volatility scaled to target 40%, cap 0.95.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The exact local-extreme formula is inside private `vivace`; notebook reports gross returns only.
- Best known candlestick timeframe(s): Daily futures data.
- reported_metrics: none reported
- notes: No Donchian period is inferred from the private signal class.
- sources: [rolling-panda-san/notebooks — BreakoutLocalExtreme portfolio and volatility-scaling call; code cell 7](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/trend_following_breakout.ipynb)

### G086 | MACD-Cross + 10-Bar S/R Break Scalper (XAU/USD M1)
- priority: Low — Intraday scalper; the MACD+breakout confluence idea transfers but the parameters don't.
- direction: long/short | market: XAU/USD (MT5) | timeframe: M1 (intraday-only)
- indicators: MACD cross; dynamic 10-bar support/resistance
- Overview: Gold scalper: buy when MACD crosses above signal AND price breaks above the recent (10-bar) resistance; sell on MACD cross down through 10-bar support. Momentum + level-confirmation confluence; better during London session per author.
- Stock or ETF Criteria: XAU/USD.
- Stocks to avoid and known failure conditions: M1 gold noise; session-dependent (London).
- Entry conditions: MACD line crosses above signal AND price > last 10-bar resistance -> buy; mirror for sell.
- Exit conditions: Fixed SL 15 pips / TP 10 pips.
- Trade management: Fixed pip exits.
- Target: +10 pips.
- Risk factors and risk management: -15 pip SL (negative skew R:R).
- Best known candlestick timeframe(s): M1; a daily analog = MACD-cross + 10-day-high breakout confluence (testable).
- reported_metrics: none reported
- notes: Negative R:R (15 SL / 10 TP) - needs high win rate.
- sources: [3aLaee/xauusd-trading-bot — signal logic](https://github.com/3aLaee/xauusd-trading-bot/blob/6fb632982b6e6ed653c614afcc1d5aa3355cb893/bot.py); [3aLaee/xauusd-trading-bot — strategy description](https://github.com/3aLaee/xauusd-trading-bot/blob/6fb632982b6e6ed653c614afcc1d5aa3355cb893/README.md)

### G087 | Median-Renko Bar-Reversal EA
- priority: Low — The code makes orders on confirmed Renko reversals, but the signal generator is external and backtest evidence is not supplied.
- direction: long/short | market: Instrument supplied by MetaTrader chart | timeframe: Renko bars (intraday-adaptable)
- indicators: Renko reversal signal; optional MA1/MA2/MA3 and SuperTrend filters
- Overview: The EA enters on a Renko bar reversal and can optionally filter entries with moving averages or a SuperTrend channel. It is a configurable trend/reversal system rather than a fixed daily-bar rule.
- Stock or ETF Criteria: Requires an external Renko chart indicator and MetaTrader broker data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Open after one confirming Renko reversal signal by default; optional MA and SuperTrend filters are off by default.
- Exit conditions: Close on reversal by default, with 200-point stop and 400-point target. Stop-and-reverse is disabled by default.
- Trade management: Default 0.1 lot, 1-bar entry/exit confirmation, 200-point stop and 400-point target; optional trading window 09:00–16:00.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Renko-native; the source does not specify a daily-bar adaptation.
- reported_metrics: none reported
- notes: The attached Renko indicator's box construction and optional filters are not included in this repository.
- sources: [9nix6/Median-and-Turbo-Renko-indicator-bundle — Default reversal confirmation, stop/target, filter and reversal settings](https://github.com/9nix6/Median-and-Turbo-Renko-indicator-bundle/blob/e0781e2a405fb45fcc195bc29f47bf7abb12e12d/Experts/Renko_EA.mq5#L58-L100)

### G088 | Median-Renko Price/Moving-Average Crossover
- priority: Low — The signal is fully implemented but its MA period is external/configurable and no robust backtest is supplied.
- direction: long/short | market: Instrument supplied by MetaTrader chart | timeframe: Custom chart bars; default trading schedule 09:00–17:55 (intraday-adaptable)
- indicators: Close-to-moving-average cross with configurable confirmation bars
- Overview: Price crossing its moving average supplies the directional signal, with optional multi-bar confirmation and session restrictions. The EA's default exit is an opposing signal.
- Stock or ETF Criteria: Requires the external MedianRenko chart indicator.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Enter long when price crosses above the moving average and short when it crosses below; default confirmation and prior-signal bars are each 1.
- Exit conditions: Close on the opposing signal by default; stop/target are disabled by default.
- Trade management: Default lots 0.1; trading window 09:00–17:55; optional stop/target, trailing and partial-close controls.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Custom chart bars; daily mapping is not specified.
- reported_metrics: none reported
- notes: The external indicator implementation is not included in the clone.
- sources: [9nix6/Median-and-Turbo-Renko-indicator-bundle — Price/MA signal, confirmation, exit, and session configuration](https://github.com/9nix6/Median-and-Turbo-Renko-indicator-bundle/blob/e0781e2a405fb45fcc195bc29f47bf7abb12e12d/Experts/PriceMA_Cross.mq5#L82-L105)

### G089 | Median-Renko SuperTrend Reversal EA
- priority: Low — The EA has concrete trade execution and risk parameters, but its primary signal implementation is an external indicator and is Renko-specific.
- direction: long/short | market: Instrument supplied by MetaTrader chart | timeframe: Renko bars (intraday-adaptable)
- indicators: External Renko SuperTrend channel
- Overview: The EA trades a SuperTrend channel direction change on a Median-Renko chart, holding one position at a time. The actual indicator and channel calculation are external to the repository.
- Stock or ETF Criteria: Requires the proprietary AZ-INVEST Median-Renko/SuperTrend chart indicator and a broker symbol.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: On a completed Renko bar, enter long/short when the SuperTrend signal changes to the corresponding direction; exact channel parameters are not specified in source.
- Exit conditions: The example EA uses 200-point stop loss and 400-point take profit; it does not state a maximum hold.
- Trade management: One open trade at a time; default lot 0.1, SL 200 points, TP 400 points.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Uses Renko bars; mapping to daily bars is not specified.
- reported_metrics: none reported
- notes: `ExampleEA.mq5` and `ExampleEA3.mq5` only demonstrate indicator buffers/log output; they do not implement trading logic.
- sources: [9nix6/Median-and-Turbo-Renko-indicator-bundle — Renko SuperTrend signal dispatch and one-position rule](https://github.com/9nix6/Median-and-Turbo-Renko-indicator-bundle/blob/e0781e2a405fb45fcc195bc29f47bf7abb12e12d/Experts/ExampleEA2.mq5#L147-L220)

### G090 | Median-Renko Two-Moving-Average Crossover
- priority: Low — The implementation is explicit but uses configurable external MA inputs and has no cost-aware backtest.
- direction: long/short | market: Instrument supplied by MetaTrader chart | timeframe: Custom chart bars; default trading schedule 09:00–17:55 (intraday-adaptable)
- indicators: Fast and slow moving-average crossover on custom/Renko chart bars
- Overview: A configurable two-moving-average crossover trades the direction of the cross and can close or reverse on the opposing cross. It is a conventional trend-following rule packaged for nonstandard chart types.
- Stock or ETF Criteria: Requires the external MedianRenko chart indicator.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Enter long/short on the custom indicator's moving-average crossover; periods and applied price are configured in the external MedianRenko indicator and are not specified in the EA source.
- Exit conditions: Close or reverse on the opposite cross; default stop loss 100 points and take profit 300 points.
- Trade management: Default lots 0.1; stop 100 points, target 300 points; trading window 09:00–17:55.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Renko/custom chart bars; daily mapping is not specified.
- reported_metrics: none reported
- notes: The indicator settings required to reproduce the MA periods are not part of this repository.
- sources: [9nix6/Median-and-Turbo-Renko-indicator-bundle — Entry/validation crossover dispatch, order settings, and session defaults](https://github.com/9nix6/Median-and-Turbo-Renko-indicator-bundle/blob/e0781e2a405fb45fcc195bc29f47bf7abb12e12d/Experts/2MA_Cross.mq5#L84-L104)

### G091 | Parabolic SAR Stop-and-Reverse Trend
- priority: Low — Signal rules are straightforward but the code is a basic indicator demonstration without a cost-aware test.
- direction: long/short | market: Forex and commodity examples | timeframe: 1d (daily-native)
- indicators: Parabolic SAR with acceleration factor
- Overview: The code computes Wilder-style Parabolic SAR recursively and holds long when SAR is below close or short when SAR is above close. The SAR reversal is both the trend signal and the stop/reversal trigger.
- Stock or ETF Criteria: OHLC time series; source notes forex/commodity acceleration defaults.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Initialize acceleration factor 0.02, increment 0.02 to cap 0.2; enter long when real SAR falls below close and short when it rises above close.
- Exit conditions: Reverse/exit when SAR crosses the close to the opposite side.
- Trade management: Stop-and-reverse exposure; no position sizing or transaction costs are specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are used in the example; other bar sizes are possible.
- reported_metrics: none reported
- sources: [je-suis-tm/quant-trading — Parabolic SAR calculation and position signal](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Parabolic%20SAR%20backtest.py#L30-L101)

### G092 | Rayner-Teo Moving-Average Pullback Strategy (RT_MA)
- priority: Low — Mainstream pullback-in-trend system overlapping S087; exact parameters in the strategy doc.
- direction: long/short | market: FX (MT4) | timeframe: chart TF (param) (intraday-adaptable)
- indicators: Moving averages (trend filter + pullback reference) per Rayner Teo MA strategy doc
- Overview: Implementation of the Rayner Teo moving-average pullback strategy: trend filter via MA, entry on pullback toward the MA with momentum resumption, ATR-based management. Faithful port of a widely-taught discretionary swing strategy.
- Stock or ETF Criteria: FX pairs.
- Stocks to avoid and known failure conditions: not specified in source. MA pullback strategies suffer in ranges. Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: not specified in source. Pullback-to-MA in trend direction per the included strategy doc.
- Exit conditions: not specified in source.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily/4h per the teaching material.
- reported_metrics: none reported
- notes: Overlap S087.
- sources: [matthewkastor/Metatrader — EA](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/Rayner%20Teo%20Moving%20Avg%20Strategy%202018_01_01/RT_MA_Pullback_Strategy.mq4); [matthewkastor/Metatrader — strategy doc](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/Rayner%20Teo%20Moving%20Avg%20Strategy%202018_01_01/Rayner%20Teo%20Moving%20Avg%20Trading%20Strategy%202018_01_03.md)

### G093 | SMA Crossover Trend (50/200; grid-optimized pairs)
- priority: Low — Golden cross is already in the catalog family (S089); value is the explicit overfitting/forward-test methodology.
- direction: long/short | market: FX EUR/USD daily | timeframe: 1d (daily-native)
- indicators: SMA_S vs SMA_L crossover; +1/-1 positions
- Overview: Canonical golden/death-cross: long when SMA_S > SMA_L, short otherwise. Notebook brute-force optimizes the pair (found 46/137 best on-sample) and explicitly documents the overfitting trap and the need for forward testing.
- Stock or ETF Criteria: EUR/USD daily; any trending instrument.
- Stocks to avoid and known failure conditions: Choppy regimes whipsaw; optimized pairs overfit (46/137 was data-mined on-sample).
- Entry conditions: Long when SMA_S > SMA_L else short, shifted one bar.
- Exit conditions: Cross reversal.
- Trade management: +1/-1 unit; costs modeled proportionally.
- Target: Ride trend to next cross.
- Risk factors and risk management: Slow exits on reversals; drawdown in ranges.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: Notebook documents 50/200 underperforming buy-and-hold pre-2015 (repo-claimed, unverified).
- notes: Overlap S089.
- sources: [BearsOnMars/Algorithmic_Trading — 50/200 + 46/137 optimization](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Moving_Average_Crossover/SMA%20Strategies%20-%20Design%20and%20Backtest.ipynb); [BearsOnMars/Algorithmic_Trading — backtester class](https://github.com/BearsOnMars/Algorithmic_Trading/blob/5a1636fdd24f843aef5531cc8e1764a6e9fd6424/Trading_Strategies/Moving_Average_Crossover/SMA%20Backtester%20Class%20with%20OOP.ipynb)

### G094 | Simple-Moving-Average MACD Oscillator Crossover
- priority: Low — The rule is precise at the moving-average level but is a generic tutorial crossover without a verified edge.
- direction: long/short | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: Fast and slow moving averages of close; oscillator difference
- Overview: This notebook defines a MACD-like oscillator from configurable moving averages and holds long while the fast average is above the slow average, otherwise flat/short according to the plotted signal implementation.
- Stock or ETF Criteria: Daily close prices from Yahoo Finance.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: After the warmup, set position to 1 when MA1≥MA2 and 0 otherwise; emit the first difference as the trade signal. The oscillator is MA1−MA2; actual MA periods are configured outside the function.
- Exit conditions: Exit long on the fast-below-slow cross; the code does not specify a fixed target or stop.
- Trade management: Example portfolio uses fixed share count and starting cash; transaction costs are not specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The code calls this MACD but uses simple moving averages in the implementation.
- sources: [je-suis-tm/quant-trading — Moving-average crossover signal and oscillator](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/MACD%20Oscillator%20backtest.py#L21-L57)

### G095 | Singular-Spectrum-Analysis Trend Signal
- priority: Low — The signal is implementable on daily data but its latent-dimension choices and empirical edge are not independently established.
- direction: long/short | market: Multi-asset dataset; exact asset list not specified in source | timeframe: Daily signals, approximately monthly rebalance (daily-native)
- indicators: Standardized price history, first principal component, volatility/risk-parity weights
- Overview: SSA reconstructs the dominant component of each asset's standardized price series and uses its latest component change as a signed trend signal. A portfolio-level volatility target and optional trend-quantity filter control exposure.
- Stock or ETF Criteria: Daily multi-asset price and return histories; the dataset universe is not specified in source.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: At each 21-bar allocation date, standardize the trailing price series, build the lagged trajectory matrix, extract the first eigencomponent, and use its last-versus-first component change rescaled to the configured range.
- Exit conditions: Recompute positions monthly; no fixed price-based exit is specified.
- Trade management: Main configuration tests volatility parity, risk parity, equal weight, and 50% overall/individual trend thresholds; default target volatility is 10%.
- Target: No fixed target; portfolio is re-evaluated at each monthly rebalance.
- Risk factors and risk management: SSA component estimates depend on the selected trajectory length and latent dimension; no transaction-cost-aware OOS result is cited.
- Best known candlestick timeframe(s): Daily data are native; monthly allocation is slower than the user's multiple-trades-per-week target.
- reported_metrics: none reported
- notes: The source is a trend-following method; `SSA_Signal` returns the normalized change in the first component, not a pairwise spread signal.
- sources: [blacksouane/QARMII_project — First-component trend extraction and rescaling](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Signal/SSA_Signal.m#L36-L64); [blacksouane/QARMII_project — 21-bar allocation, signal, risk weights, and 10% volatility target](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/SSA/SSA_TF.m#L121-L247); [blacksouane/QARMII_project — SSA EW, risk-parity and trend-quantity variants](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Main.m#L525-L584)


## Momentum, Rotation and Cross-Sectional Factors

### G096 | ANTICOR Anti-Correlation Wealth Transfer
- priority: High — Published, distinctive cross-sectional lead-lag mechanism with strong OLPS benchmark results; daily-native.
- direction: long | market: equity index constituents | timeframe: 1d (daily-native)
- indicators: Log-price windows; cross-correlation between consecutive window blocks
- Overview: Borodin's ANTICOR: split the recent price history into two equal windows, compute the cross-correlation of log-returns between stocks' recent and previous windows, and transfer portfolio weight from stocks whose recent gains were predicted by other stocks' earlier moves to the 'loser' stocks expected to catch up. Exploits cross-asset lead-lag/reversal - consistently among the top OLPS performers and unlike anything in the existing catalog.
- Stock or ETF Criteria: Multi-stock daily panel (the algorithm needs cross-asset correlation; ~20+ names).
- Stocks to avoid and known failure conditions: Fails when lead-lag structure is unstable; daily rebalancing turnover costs; window too small = noisy correlations.
- Entry conditions: Compute cross-correlation between last-w and prior-w log-return blocks across stocks; shift weight proportional to (anti-correlation score) toward lagging stocks each day.
- Exit conditions: Daily rebalance (weights define position).
- Trade management: Long-only simplex weights; bounded per-step transfer fraction.
- Target: Exploit cross-stock lead-lag reversal for daily compounding.
- Risk factors and risk management: Weight-transfer rule can concentrate into recent losers (falling knives) - cap per-name weight.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- external_refs: Borodin, El-Yaniv & Gogan, 'Can we learn to beat the best stock' (ANTICOR)
- notes: Also implemented in other OLPS repos; merge sources if another clone has it.
- sources: [MXGao-A/TradeRules — ANTICOR implementation](https://github.com/MXGao-A/TradeRules/blob/964e04a11d65bc66e8ef88ae115cb08bc006c080/pgportfolio/tdagent/algorithms/anticor1.py); [MXGao-A/TradeRules — variant](https://github.com/MXGao-A/TradeRules/blob/964e04a11d65bc66e8ef88ae115cb08bc006c080/pgportfolio/tdagent/algorithms/anticor2.py)

### G097 | Baltas 2020 Trend Signal with Realized-Volatility Scaling
- priority: Medium — Risk estimator and signal are reproducible at the notebook-call level, but private code and gross returns limit confidence.
- direction: long/short | market: Global futures | timeframe: Daily futures; month-end rebalance (daily-native)
- indicators: Baltas 2020 trend signal; 21-session volatility estimator
- Overview: This notebook compares the original trend signal, volatility-scaling variants, and the Baltas 2020 trend signal. It targets a 40% annualized portfolio volatility and evaluates turnover reduction.
- Stock or ETF Criteria: Diversified global futures universe in the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use `TSMOMBaltas2020()` with 21-session rolling or Yang–Zhang realized volatility (min_periods=10), target volatility 0.4, and cap 0.95; carry the signal to month-end.
- Exit conditions: Rebalance at month-end; no fixed price stop or target is specified.
- Trade management: Equal weight assets; compare rolling/EWM/Yang–Zhang volatility scaling, target 40%, cap 0.95.
- Target: Monthly portfolio rebalance.
- Risk factors and risk management: Backtests are gross of costs; full signal implementation is private.
- Best known candlestick timeframe(s): Daily futures data; monthly rebalance.
- parameter_variants: 21-session rolling, EWM, and Yang–Zhang realized-volatility scaling.
- reported_metrics: none reported
- external_refs: Baltas and Kosowski (2020), as cited in the notebook.
- sources: [rolling-panda-san/notebooks — Original, volatility-scaled, Baltas-signal, and combined portfolios; code cells 8, 13, 18, and 23](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/trend_following_baltas2020.ipynb)

### G098 | Moskowitz-Style Time-Series Momentum on Futures
- priority: Medium — Time-series momentum is a well-motivated futures premium and daily-adaptable, but net costs and code-level signal details are unavailable.
- direction: long/short | market: Global futures, including Chinese commodity futures | timeframe: Daily futures; month-end rebalance (daily-native)
- indicators: Moskowitz 2012 time-series momentum and volatility scaling
- Overview: A diversified futures portfolio combines the repository's Moskowitz 2012 time-series momentum signal with volatility scaling. A Chinese commodity notebook applies the same signal approach to a local futures universe.
- Stock or ETF Criteria: Diversified global futures; source describes about 50 contracts, plus a separate Chinese commodity universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use private `TSMOMMoskowitz2012` and EWM volatility scaling (com=60, annualization=261, target volatility=0.4, signal cap=0.95); carry positions to month-end.
- Exit conditions: Rebalance at month-end; no price-based stop or profit target is specified.
- Trade management: Equal weight across instruments; EWM vol scale target 40%, cap 0.95; gross backtests (`calculate_net=False`).
- Target: Monthly rebalance; no fixed price target.
- Risk factors and risk management: The notebooks report gross returns and note prolonged drawdowns; signal implementation is private.
- Best known candlestick timeframe(s): Daily futures bars are native; monthly position refresh.
- reported_metrics: none reported
- external_refs: Moskowitz, Ooi, and Pedersen (2012), as cited in the notebook.
- notes: The code identifies the signal class but does not include the private `vivace` implementation.
- sources: [rolling-panda-san/notebooks — TSMOM and EWM volatility scaling on the global futures universe; code cell 7](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/trend_following_moskowitz2012.ipynb); [rolling-panda-san/notebooks — Same signal on Chinese futures with month-end rebalance; code cell 10](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_trend_following_chinese_futures.ipynb); [rolling-panda-san/notebooks — Moskowitz TSMOM comparison portfolio; code cell 10](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/trend_following_breakout.ipynb)

### G099 | Online Portfolio Selection Algorithm Zoo (OLMAR/PAMR/CWMR/RMR/ANTICOR/CORN/UP/ONS...)
- priority: Medium — Rich source of published daily-rebalance algorithms; OLMAR/ANTICOR merit direct porting; others likely cost-limited.
- direction: long | market: NASDAQ100/DOW30/SSE50/CSI300 + crypto (pgportfolio) | timeframe: 1d (daily-native)
- indicators: Price relatives; mean-reversion predictors (SMA reversion for OLMAR); anti-correlation matrices; pattern-matching kernels
- Overview: Implements the classic OLPS (online portfolio selection) algorithm suite on daily bars: mean-reversion family (OLMAR - price reverts to moving average; PAMR/CWMR/RMR - passive-aggressive and confidence-weighted MR), trend/momentum family (follow-the-winner/loser, UP/EG/ONS universal portfolios), and pattern-matching (BK kernel, BNN, CORN correlation-driven). Each is a full daily rebalance rule with published academic results - OLMAR and ANTICOR are the standout performers on equity index datasets in the OLPS literature.
- Stock or ETF Criteria: Index-constituent or multi-asset panels; works best on ~10-100 liquid names with daily closes.
- Stocks to avoid and known failure conditions: MR family assumes daily reversal - fails in strong trending regimes; costs are the killer at daily rebalance frequency on retail spreads; CRP/UBAH are baselines not strategies.
- Entry conditions: Per-algorithm daily weight update (e.g., OLMAR: predicted next price = SMA(w) of recent prices; weight toward underperformers when expected MR return > epsilon).
- Exit conditions: Full rebalance each session to new weights (no discrete exit - weights ARE the position).
- Trade management: Weights sum to 1, long-only, no cash leg by default; transaction costs modeled in backtest.
- Target: Maximize cumulative wealth vs CRP benchmark.
- Risk factors and risk management: Daily full rebalancing = high turnover; edge depends on daily cross-sectional reversal persisting after costs.
- Best known candlestick timeframe(s): Daily bars (native to the OLPS literature and code).
- reported_metrics: README lists backtest harness; OLPS survey (Li & Hoi) reports OLMAR/ANTICOR dominating on NYSE/TSE datasets (literature, unverified here).
- external_refs: Li & Hoi, 'Online Portfolio Selection: A Survey'; Blum & Kalai OLMAR (2006); Gyorfi ANTICOR
- notes: Vendored pgportfolio tree (RL portfolio agent) also present - separate approach.
- sources: [MXGao-A/TradeRules — 19 OLPS algorithms: olmar.py, pamr.py, cwmr_std.py, rmr.py, anticor1.py, cornk.py, bk.py, bnn.py, up.py, ons.py, eg.py, ubah.py, crp.py](https://github.com/MXGao-A/TradeRules/blob/964e04a11d65bc66e8ef88ae115cb08bc006c080/pgportfolio/tdagent/algorithms/); [MXGao-A/TradeRules — feature list](https://github.com/MXGao-A/TradeRules/blob/964e04a11d65bc66e8ef88ae115cb08bc006c080/README.md)

### G100 | QARMII 252-Day Binary Momentum Portfolio
- priority: Medium — The 12-month direction rule is explicit and diversified, but the source presents a research portfolio rather than a verified modern out-of-sample implementation.
- direction: long/short | market: Multi-asset dataset; exact asset list not specified in source | timeframe: Daily signals, approximately monthly rebalance (daily-native)
- indicators: 252-day cumulative returns, 180-day covariance/volatility
- Overview: This portfolio takes the sign of each asset's trailing 252-day return and combines the cross-asset signals with volatility-parity or equal-weight allocations. It is a time-series momentum baseline with target-volatility leverage.
- Stock or ETF Criteria: Assets with daily return history; the repository's dataset spans multiple asset classes.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Every 21 daily observations, set each asset signal to +1 when its trailing 252-day cumulative return is positive and −1 otherwise; the script's binary-signal helper implements the direction.
- Exit conditions: Rebalance on the next 21-bar signal date; no discretionary exit is specified beyond the sign change.
- Trade management: Default configuration uses a 10% annualized volatility target and 180-day volatility window; equal-weight, volatility-parity, and risk-parity schemes are tested. Portfolio return code deducts 0.1% costs.
- Target: No fixed price target; signal is recomputed at the monthly rebalance.
- Risk factors and risk management: Binary signs can create turnover around zero cumulative return; universe composition and data period are not specified in source.
- Best known candlestick timeframe(s): Daily bars are native; monthly allocation updates suit swing/position trading, not multiple trades per week.
- reported_metrics: none reported
- notes: The signal is the canonical 252-day binary momentum approach; it is not the catalog's 12-1 cross-sectional winner sort.
- sources: [blacksouane/QARMII_project — Monthly signal loop and binary momentum branches](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/MOM_MA/model1.m#L145-L270); [blacksouane/QARMII_project — Binary sign-of-return signal](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Others/binsignal.m); [blacksouane/QARMII_project — Momentum252 portfolio configuration and 0.1 volatility target](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Main.m#L69-L82)

### G101 | Quadratic Dip-Depth Sizing Formula on VTI/QQQ (SGOV cash park)
- priority: Medium — Novel sizing-as-function-of-depth mechanic worth testing vs fixed DCA and vs RSI-gated dip buys; slow frequency.
- direction: long | market: VTI, QQQ (cash parked in SGOV) | timeframe: 1d (daily-native)
- indicators: 50-day high distance (dip%) on daily closes
- Overview: A continuous dip-buying formula rather than a threshold signal: daily target deployment = 5*dip% + dip%^2 (quadratic in depth below the 50-day high), and today's buy = max(0, target - already deployed this cycle). Idle cash sits in SGOV (T-bill ETF). Converts drawdown depth directly into position size - deep dips buy aggressively; a mechanical way to harvest the equity risk premium through drawdowns.
- Stock or ETF Criteria: VTI and QQQ only; SGOV as the cash vehicle.
- Stocks to avoid and known failure conditions: A secular bear (2008, 2022) keeps adding into a deepening dip - deployment can exhaust capital; quadratic term is aggressive.
- Entry conditions: Daily: dip% = max(0, 100*(1 - price/50d_high)); target_total = 5*dip% + dip%^2; buy max(0, target_total - deployed_in_cycle).
- Exit conditions: Not specified in source (accumulation-style; cycle reset on new highs implied). Suggested: reset deployed_in_cycle when price makes a new 50-day high.
- Trade management: Continuous sizing vs fixed dollar DCA; $500/month capital input.
- Target: Recovery to/above the 50-day high.
- Risk factors and risk management: No stop-loss by design; concentration in 2 correlated ETFs.
- Best known candlestick timeframe(s): Daily bars (native).
- reported_metrics: none reported
- notes: ai-value-chain.example.md is a plain monthly DCA on a curated watchlist - no edge, logged not recorded.
- sources: [FarzamHejaziK/claude-investment-assistant — dip formula](https://github.com/FarzamHejaziK/claude-investment-assistant/blob/9607f6b52214f20e340c1c27491766b78b7acc78/strategies/dip-buying.example.md)

### G102 | Top-50 S&P500 12-Month Momentum Portfolio (with HQM low-quality-momentum filter variant)
- priority: Medium — Crossref to S041 (12-1 cross-sectional momentum); the HQM multi-horizon filter is a practical quality gate worth registering as a variant.
- overlap: S041 (existing catalog) — Basic version uses raw 12m return (includes most recent month, unlike 12-1 academic). HQM variant = mean of 4 horizon percentiles - worth adding as a parameter variant of S041.
- direction: long | market: S&P 500 constituents (IEX) | timeframe: 1d (monthly rebalance) (daily-native)
- indicators: 1-year price return percentile; HQM variant averages 1y/6m/3m/1m return percentiles and drops weak-subperiod names
- Overview: freeCodeCamp course strategy: rank S&P 500 by trailing 12-month price return, buy the top 50 equal-weight, monthly rebalance. The 'high-quality momentum' variant requires strength across multiple lookback horizons (mean of 1y/6m/3m/1m percentiles), dropping names whose momentum is driven by a single stale window.
- reported_metrics: none reported
- external_refs: freeCodeCamp Algorithmic Trading in Python course
- sources: [nickmccullum/algorithmic-trading-python — top-50 by 1y return; HQM multi-horizon percentile](https://github.com/nickmccullum/algorithmic-trading-python/blob/e1e3c2e586188a9222d3de61849cfeef29d3d28a/finished_files/002_quantitative_momentum_strategy.ipynb)

### G103 | Top-One Positive-Slope Momentum Rotation
- priority: Medium — The signal is precise, daily-native, and the notebook includes walk-forward windows, but concentrated top-one exposure and index-selection leakage risk remain.
- direction: long | market: US equities; Nasdaq-100 constituents and selectable global equity indices | timeframe: 1d (daily-native)
- indicators: 250-session log-price slope and 250-session price ROC
- Overview: The engine selects the instrument with the highest recent log-price slope, gated by a positive 250-session return, and switches the portfolio to that single winner. This is a concentrated, long-only cross-sectional trend rotation.
- Stock or ETF Criteria: Universe is NASDAQ-100 constituents or one selected index universe; source uses adjusted closes from Yahoo Finance.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Every 14 calendar days, score each instrument with `ceil(250-session ROC in percent)` clipped to [0,1], multiplied by the least-squares slope of log prices over 250 bars; buy the top scorer if its score is positive.
- Exit conditions: Sell any held instrument that is no longer the top-ranked instrument and buy the new top scorer. Liquidate on the last bar when configured.
- Trade management: Top-1 position, 95% of equity, rebalanced every 14 calendar days; a fixed $7 transaction cost is configured.
- Target: No fixed profit target; rotate at the next rebalance.
- Risk factors and risk management: No explicit stop. In the notebook's optional best-index mode, the selected index can be chosen using the same full evaluation window, creating look-ahead selection risk.
- Best known candlestick timeframe(s): Daily bars are native; the 250-session signal is not intended for intraday bars.
- parameter_variants: Walk-forward grid tests lookback/ROC 150–250 sessions, 7–21 calendar-day rebalance, and 95% exposure.
- reported_metrics: none reported
- notes: The 250-session ROC is rounded up and clipped to [0,1], so for most positive-return assets the score effectively becomes the log-price slope.
- sources: [0a1b/momentum_NautilusTrader — Daily scoring, top-N selection, rebalance schedule, and position switch](https://github.com/0a1b/momentum_NautilusTrader/blob/31d29628fe7c770f1b8fd07a598658062cd522b1/nautilus_engine_momentum.py#L136-L184); [0a1b/momentum_NautilusTrader — Top-one portfolio liquidation and sizing](https://github.com/0a1b/momentum_NautilusTrader/blob/31d29628fe7c770f1b8fd07a598658062cd522b1/nautilus_engine_momentum.py#L190-L248); [0a1b/momentum_NautilusTrader — Notebook parameters and candidate index selection, code cells 1 and 2](https://github.com/0a1b/momentum_NautilusTrader/blob/31d29628fe7c770f1b8fd07a598658062cd522b1/momentum_top1_nasdaq100_nautilus_v5.ipynb); [0a1b/momentum_NautilusTrader — Walk-forward parameter grid and windows, code cell 2](https://github.com/0a1b/momentum_NautilusTrader/blob/31d29628fe7c770f1b8fd07a598658062cd522b1/momentum_walkforward_runner.ipynb)

### G104 | 232-Session-Lagged Industry-Relative Momentum Factor
- priority: Low — The signal is precise but the code's academic interpretation is ambiguous and no cost-aware OOS evidence is provided.
- direction: long/short | market: US equities | timeframe: Daily (daily-native)
- indicators: 20-session cumulative log return, industry demeaning, 232-session lag
- Overview: The factor applies a 232-session lag to a 20-session cumulative return, aiming to isolate longer-term continuation/reversal effects outside the most recent short-term window. The signal is demeaned within industries and fit to forward returns.
- Stock or ETF Criteria: US equities with daily prices and industry classifications; at least 252 sessions of history.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Calculate a 20-session cumulative log return, demean within date/industry, shift the factor by 232 sessions, then use the in-sample regression coefficient to form daily forecasts.
- Exit conditions: Forecast alpha each day; no discrete exit or portfolio holding period is specified.
- Trade management: Forecast is consumed by the daily portfolio optimizer.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The module docstring describes this as annual momentum while also relating it to long-term reversal; do not infer a desired coefficient sign.
- sources: [noterminusgit/statarb — Signal horizon, 232-session lag, and forecast model](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/mom_year.py#L50-L151)

### G105 | Beta-Divided Multi-Period Return Reversal Factor
- priority: Low — The factor is explicit and daily, but its regression targets and data dependencies require substantial replication work.
- direction: long/short | market: US equities | timeframe: 1d (daily-native)
- indicators: Log return divided by predicted beta, winsorized and industry demeaned
- Overview: This factor normalizes each stock's return by predicted beta and fits multiple lagged beta-adjusted returns to forward returns. Its daily/overnight component is intended to capture stock-specific continuation or reversal after removing broad market exposure.
- Stock or ETF Criteria: US equities with daily returns, predicted beta, market cap, and industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Set o2c0=log_return/predicted_beta, winsorize by date, demean within industry, create lagged o2c1..o2cH, and apply the in-sample regression coefficients.
- Exit conditions: Recompute the cross-sectional forecast daily; no discrete entry/exit threshold is specified.
- Trade management: Forecasts are consumed by the daily bsim optimizer; no direct per-name stop or target.
- Target: Default forecast horizon and lag count are not specified in source.
- Risk factors and risk management: Beta division is unstable for near-zero beta; the strategy requires point-in-time Barra data and realistic turnover costs.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The repository labels this `badj_multi`; despite the name, it is return/beta rather than order-flow alpha.
- sources: [noterminusgit/statarb — Beta-divided return construction, lags, and regression forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj_multi.py#L129-L421)

### G106 | Cross-Sectional 252-Session Commodity Momentum
- priority: Low — The one-year factor is interpretable but ranks and costs are unavailable in the clone.
- direction: long/short | market: Commodity futures | timeframe: 1d (daily-native)
- indicators: 252-session futures momentum
- Overview: A cross-sectional commodity-momentum signal ranks futures on a one-year lookback and holds the strongest/weakest signal exposures. Exact weighting and trade timing are delegated to the private signal library.
- Stock or ETF Criteria: Hollstein 2020 commodity futures universe.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `XSMomentum(lookback=252)` on the Hollstein 2020 futures universe; rank cutoffs and directional mapping are not specified in source.
- Exit conditions: Signal is refreshed by the backtest engine; explicit exit and rebalance cadence are not specified in the notebook.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Signal implementation and transaction costs are inside private `vivace`; notebook plots gross returns.
- Best known candlestick timeframe(s): Daily futures data are used; exact rebalance frequency is not specified.
- reported_metrics: none reported
- external_refs: Asness, Moskowitz, and Pedersen (2013), as cited in the repository README.
- sources: [rolling-panda-san/notebooks — 252-session XSMomentum call and futures universe; code cell 7](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_momentum.ipynb)

### G107 | Cross-Sectional Commodity Basis Momentum
- priority: Low — The basis-momentum thesis is economically plausible, but signal thresholds and net trading costs are not exposed in the clone.
- direction: long/short | market: Commodity futures | timeframe: Daily futures data; signal implementation private (daily-native)
- indicators: Futures basis across first and second expiries
- Overview: The notebook applies a cross-sectional momentum signal to futures basis, both as outright commodity exposure and as a calendar-spread portfolio. The thesis is persistence in futures basis/roll-yield components.
- Stock or ETF Criteria: Boons 2019 commodity futures universe; first two expiries in the spread variant.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `XSBasisMomentum()` on the Boons 2019 futures universe with 21-session contract rolls and at most the first two expiries; a spread variant constructs first/second-expiry legs. Internal ranking and cutoffs are not specified in source.
- Exit conditions: Recompute the signal through the backtest engine; exact position exit/holding rule is not specified in source.
- Trade management: Contracts roll on a 21-session schedule; the notebook plots gross equity curves (`calculate_net=False`).
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The signal implementation is in the private `vivace` library; exact lookback/ranks/costs are unavailable.
- Best known candlestick timeframe(s): Daily futures bars are native; the notebook examines post-publication performance.
- reported_metrics: none reported
- external_refs: Boons et al. (2019), as named in the notebook; complete citation is in its reference cell.
- notes: The README identifies Boons 2019; internal `vivace` signal logic is proprietary.
- sources: [rolling-panda-san/notebooks — XSBasisMomentum outright and XSBasisMomentumSpread calls; code cells 6 and 11](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_basis_momentum.ipynb)

### G108 | Cross-Sectional Futures Carry
- priority: Low — Carry is a structural futures risk premium, but the repository hides signal construction and shows gross backtests.
- direction: long/short | market: Commodity and FX futures | timeframe: Daily futures; monthly signal refresh (daily-native)
- indicators: Near-versus-deferred futures carry/term structure
- Overview: The strategy ranks futures on curve carry and holds the cross-sectional carry portfolio. The commodity notebook smooths carry with a 252-session SMA before month-end; the FX notebook runs the same signal class without that smoothing pipeline.
- Stock or ETF Criteria: Commodity/FX futures with an observable futures curve.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `XSCarryFutureFuture(nth_expiry_shift=1)` on commodity futures and carry an SMA252-smoothed score at month-end; the FX notebook calls `XSCarryFutureFuture()` on its futures universe. Internal ranks and cutoffs are private.
- Exit conditions: Refresh at month-end; no price-based stop or profit target is specified.
- Trade management: Delta-one futures backtest; capital weights and turnover controls are internal to `vivace`.
- Target: No fixed target; monthly carry-factor rebalance.
- Risk factors and risk management: Internal carry construction and transaction costs are unavailable; curve roll and contract handling can dominate results.
- Best known candlestick timeframe(s): Daily futures prices with month-end rebalance.
- reported_metrics: none reported
- external_refs: Koijen et al. (2018) and Deutsche Bank (2009), as named in the repository README.
- notes: The commodity notebook compares smoothed and unsmoothed carry and notes extreme winter readings in natural gas.
- sources: [rolling-panda-san/notebooks — One-expiry-shift carry, SMA252 smoothing and month-end signal carry; code cells 7 and 18](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_term_structure.ipynb); [rolling-panda-san/notebooks — FX futures carry signal call; code cell 7](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/fx_carry.ipynb)

### G109 | Cross-Sectional Skewness Risk-Premium Signal
- priority: Low — The factor is paper-motivated and daily-adaptable, but full signal logic and net results are unavailable.
- direction: long/short | market: Commodity and cross-asset futures | timeframe: Daily data; monthly signal refresh (daily-native)
- indicators: 252-session realized return skewness
- Overview: A cross-sectional skewness factor ranks futures by realized-return skewness, with the commodity variant negating the score before month-end application. The thesis is compensation for skewness preferences in futures markets.
- Stock or ETF Criteria: Hollstein commodity futures and broader futures universes in the cross-asset notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `XSSkewness(lookback=252)`; the commodity notebook applies `Negate()` and carries the signal at month-end. Exact rank cutoffs and weights are not specified in source.
- Exit conditions: Refresh at month-end; no fixed price exit is specified.
- Trade management: Portfolio weights/cutoffs are private `vivace` implementation details; the backtests use delta-one exposure.
- Target: No fixed target; monthly cross-sectional rebalance.
- Risk factors and risk management: Signal calculation is not in the clone and results are gross of costs; notebook notes post-publication deterioration for commodity skewness.
- Best known candlestick timeframe(s): Daily returns with month-end signal refresh.
- reported_metrics: none reported
- external_refs: Fernandez-Perez et al. (2018) for commodity skewness; Baltas and Salinas (2019) for cross-asset skewness, as cited in notebooks.
- sources: [rolling-panda-san/notebooks — 252-session XSSkewness factor, negative orientation and month-end carry; code cell 7](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/commodity_skewness.ipynb); [rolling-panda-san/notebooks — Cross-asset XSSkewness portfolio variants; code cells 7, 9, 11, and 13](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/cross_asset_skewness.ipynb)

### G110 | Daily Plus Intraday Beta-Adjusted Return Forecast
- priority: Low — The code clearly combines two return horizons but has microstructure turnover and external data requirements.
- direction: long/short | market: US equities | timeframe: Daily plus intraday bars (intraday-adaptable)
- indicators: Daily lagged returns and current intraday return divided by predicted beta
- Overview: The combined model uses beta-adjusted daily lags plus a current intraday return signal and estimates separate coefficients for each component. It combines short-term and overnight-to-close information.
- Stock or ETF Criteria: US equities with daily and intraday data, beta, and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute daily o2c0=log_return/predicted_beta and intraday o2cC; winsorize/industry-demean and combine their regression-fitted coefficients. The intraday fit uses six time-of-day buckets.
- Exit conditions: Recompute the alpha forecast at each intraday timestamp; discrete stop and holding-time rules are not specified.
- Trade management: Forecasts are consumed by the intraday qsim engine; direct position limits are configured externally.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday bars are required for the combined signal; the daily component alone can be separately evaluated.
- reported_metrics: none reported
- notes: Alpha generation only; simulator/execution parameters are not intrinsic to this module.
- sources: [noterminusgit/statarb — Daily+intraday beta-adjusted returns and fitted signal combination](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj_both.py#L141-L400)

### G111 | Intraday Beta-Divided Return Reversal
- priority: Low — The signal is specific but execution-sensitive and cannot be mapped to daily swing trades without rebuilding it.
- direction: long/short | market: US equities | timeframe: 15-minute bars (intraday-only)
- indicators: Intraday log return divided by predicted beta; time-of-day coefficients
- Overview: The intraday-only strategy trades beta-normalized returns with separate coefficients for six hourly periods. It omits daily lagged signals and is designed for intraday execution.
- Stock or ETF Criteria: US equities with 15-minute bars, beta and industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute intraday return/predicted-beta, winsorize and industry-demean; apply the fitted coefficient for the current time bucket.
- Exit conditions: Forecast updates through the session; explicit stops and time exits are delegated to the simulator.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only; not compatible with daily-only bars.
- reported_metrics: none reported
- notes: The module does not implement broker orders or a standalone exit rule.
- sources: [noterminusgit/statarb — Intraday-only return/beta signal and time-bucket forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj_intra.py#L133-L392)

### G112 | Intraday Market-Weighted Beta-Residual Returns
- priority: Low — It is a distinct neutralization variant, but it needs quote/bar infrastructure and has no cost-aware OOS evidence.
- direction: long/short | market: US equities | timeframe: Intraday bars (intraday-only)
- indicators: Intraday returns minus predicted beta times cap-weighted market return
- Overview: This alternative intraday model removes the market component from returns using cap-weighted beta adjustment, then forecasts from the residual signal. It is a higher-frequency counterpart to badj2_multi.
- Stock or ETF Criteria: US equities with intraday returns, market cap and predicted beta.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: At each intraday bar, subtract predicted beta × cap-weighted market return, winsorize/demean, and use six time-of-day regression coefficients.
- Exit conditions: Forecast is refreshed by timestamp; explicit position exit is delegated to the simulator.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only; daily mapping is not specified.
- reported_metrics: none reported
- notes: This factor differs from the simple return/pbeta approach in badj_intra.py.
- sources: [noterminusgit/statarb — Market-weighted intraday beta adjustment and hourly coefficients](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj2_intra.py#L201-L465)

### G113 | Market-Weighted Beta-Residual Return Factor
- priority: Low — Market-neutralized return information is plausible, but no cost-aware performance evidence is reported.
- direction: long/short | market: US equities | timeframe: 1d (daily-native)
- indicators: Market-cap-weighted market return, predicted beta, lagged beta-adjusted returns
- Overview: This variant subtracts the predicted beta times the cap-weighted market return instead of dividing each return by beta. The resulting residual-like daily signals are regressed against future returns.
- Stock or ETF Criteria: US equities with market cap, predicted beta, and daily return data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each date, calculate market return as cap-weighted mean return and set adjusted return to log_return−predicted_beta×market_return; winsorize, industry-demean, lag, and fit the forecast model.
- Exit conditions: Daily re-estimation of the forecast; a separate price-based exit is not specified.
- Trade management: Portfolio optimizer consumes the alpha forecast; direct sizing/exit rules are not specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: The signal depends on Barra beta and market-cap data and can be sensitive to weighting or missing data.
- Best known candlestick timeframe(s): Daily bars are native.
- parameter_variants: The repository also supplies daily+intraday and intraday-only versions in badj2_both/badj2_intra; those timeframes are separate records.
- reported_metrics: none reported
- notes: Distinct from badj_multi.py's simple return/beta division.
- sources: [noterminusgit/statarb — Market-weighted beta adjustment and multi-lag forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj2_multi.py#L223-L490)

### G114 | NSE Stock Scanner - Momentum/Swing Screening Signals
- priority: Low — Screening layer; signal details are standard technical filters.
- direction: long | market: NSE India equities | timeframe: 1d (daily-native)
- indicators: Momentum, swing-trading and intraday screeners + risk rules
- Overview: NSE screener that surfaces momentum and swing-trading candidates with technical criteria - a universe-selection/filter layer producing daily candidate lists for swing entries.
- Stock or ETF Criteria: NSE listed equities.
- Stocks to avoid and known failure conditions: Screener, not a complete exit system.
- Entry conditions: Screened names passing momentum/swing criteria.
- Exit conditions: Per swing rules.
- Trade management: Risk-management module included.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Merge with momentum/screener group.
- sources: [deshwalmahesh/NSE-Stock-Scanner — scanner](https://github.com/deshwalmahesh/NSE-Stock-Scanner/blob/f6f47fa7dd0e30f227b9e3a9b3bcb3569de04c18/README.md)

### G115 | Robust-Value Percentile-Composite Portfolio (PE/PB/PS/EV-EBITDA/EV-GP)
- priority: Low — Standard composite-value construction; slow cadence - useful as a factor sleeve not a swing strategy.
- direction: long | market: S&P 500 constituents (IEX) | timeframe: 1d (monthly/periodic rebalance) (daily-native)
- indicators: Percentile ranks of PE, PB, PS, EV/EBITDA, EV/GP combined into an RV (robust value) score
- Overview: Cross-sectional value factor: compute each S&P 500 name's percentile on five valuation metrics, average into a single 'robust value' score, buy the cheapest 50 equal-weight. Using a percentile composite across metrics avoids single-ratio gaming and is the classic quant-value construction.
- Stock or ETF Criteria: S&P 500; requires fundamentals feed (IEX in code); positive PE filter applied first.
- Stocks to avoid and known failure conditions: Value traps and sector concentration (composite is sector-blind); periodic rebalance means positions drift far from signal.
- Entry conditions: Rank by RV score (mean percentile of PE, PB, PS, EV/EBITDA, EV/GP); buy bottom 50 equal-weight; periodic rebalance.
- Exit conditions: Rebalance out when RV rank decays.
- Trade management: Equal-weight 50 names.
- Target: Long-term value factor premium.
- Risk factors and risk management: Factor drawdowns measured in years; no risk overlay.
- Best known candlestick timeframe(s): Daily bars, monthly/quarterly rebalance cadence.
- reported_metrics: none reported
- notes: Complement to S045 multi-factor blend.
- sources: [nickmccullum/algorithmic-trading-python — RV score percentile composite](https://github.com/nickmccullum/algorithmic-trading-python/blob/e1e3c2e586188a9222d3de61849cfeef29d3d28a/finished_files/003_quantitative_value_strategy.ipynb)

### G116 | Volatility-Ratio-Weighted Return Reversal Factor
- priority: Low — The feature is measurable but the source gives no threshold, economic validation, or cost-aware results.
- direction: long/short | market: US equities | timeframe: Daily plus intraday (intraday-adaptable)
- indicators: Log return multiplied by realized-volatility ratio
- Overview: The strategy conditions stock returns on a volatility ratio, winsorizes and industry-demeans them, and fits daily/intraday coefficients. It tests whether return reversal changes with realized volatility.
- Stock or ETF Criteria: US equities with daily/intraday returns and volatility ratio data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal = log_return × volatility_ratio; generate lagged daily and intraday features, winsorize/demean by industry, and apply regression-fitted forecasts.
- Exit conditions: No direct position exit or stop is specified; forecast flows into the portfolio engine.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily component is native; intraday forecast uses intraday bars.
- reported_metrics: none reported
- notes: The module is named `other.py`; its own docstring describes a volatility-conditioned alpha.
- sources: [noterminusgit/statarb — Volatility-ratio return interaction and signal forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/other.py#L40-L159)


## Formulaic Alpha Factor Libraries

### G117 | QLib Alpha158/Alpha360 Handcrafted Factor Libraries
- priority: High — The standard open-source daily cross-sectional factor pipeline with published benchmarks - Alpha158 features + LightGBM is a direct, testable daily strategy.
- direction: long/short | market: CN A-shares + US equities | timeframe: 1d (daily-native)
- indicators: Alpha158: 158 handcrafted price/volume expressions (KBAR, price, rolling corr/mean/std/beta/RSV/quantile/max-min/sum/ema/corr of OHLCV windows 5-60d); Alpha360: 360-day normalized OHLCV window tensor for DL
- Overview: Microsoft qlib's reference factor sets: Alpha158 computes ~158 hand-designed technical expressions over rolling 5/10/20/30/60-day windows; Alpha360 feeds the raw 60-day OHLCV window directly to deep models. The factors + qlib's benchmark harness (LightGBM/MLP/LSTM/etc. ranked by IC/ICIR) are the standard open-source daily-bar cross-sectional alpha pipeline - Alpha158 alone produces a usable daily equity ranker.
- Stock or ETF Criteria: CSI300/CSI500 (native configs); adaptable to US liquid universe.
- Stocks to avoid and known failure conditions: A-share tuned factors (limit-up/down behaviors) don't all transfer; many factors are redundant/collinear - needs feature selection.
- Entry conditions: Rank universe by model score over Alpha158 features (or DL on Alpha360); top-K equal/vol-scaled long (qlib default), optional short leg.
- Exit conditions: Daily/periodic rebalance per qlib workflow (default daily top-k).
- Trade management: Top-k portfolio; qlib's order-execution layer separate.
- Target: Cross-sectional IC-driven outperformance.
- Risk factors and risk management: Factor crowding; needs rolling retrain (qlib supports rolling workflows).
- Best known candlestick timeframe(s): Daily bars (native).
- reported_metrics: none reported
- external_refs: Yang et al., Qlib: An AI-oriented Quantitative Investment Platform (arXiv:2009.11189)
- notes: qlib benchmark tables report IC/ICIR per model - Alpha158+LightGBM is the baseline to beat.
- sources: [microsoft/qlib — Alpha handlers + benchmark configs (+4 more verified paths)](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/qlib/contrib/data/handler.py); [microsoft/qlib — qlib/contrib/data/loader.py](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/qlib/contrib/data/loader.py); [microsoft/qlib — examples/benchmarks/LightGBM/workflow_config_lightgbm_Alpha158.yaml](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/examples/benchmarks/LightGBM/workflow_config_lightgbm_Alpha158.yaml)

### G118 | Alpha101 Formulaic Alpha Library
- priority: Low — The code provides many precise daily factors but no independently verified, cost-aware out-of-sample evidence for the library as a whole.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Daily OHLCV, VWAP, returns, rolling ranks, correlations, and time-series operators
- Overview: A 101-factor formula library transforms price, volume, and VWAP relationships into cross-sectional alpha scores. The repository computes factors and evaluates their forward-return quantiles; any edge depends on point-in-time data, factor selection, and realistic portfolio turnover.
- Stock or ETF Criteria: Universe and data fields vary by factor; examples use Chinese equity assets, including HS300 constituents.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Formula values are calculated per asset and date; the library itself specifies no universal entry threshold or side. The accompanying backtest selects factor quantile 5 from five quantiles for the illustrated Alpha101 factor.
- Exit conditions: No factor-specific exit is specified; the notebook's portfolio example replaces holdings on each factor date.
- Trade management: The example assigns equal weights of 1/60 to selected assets and rebalances on factor dates; transaction-cost treatment is not specified in the cited cells.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; applicability is factor-specific.
- reported_metrics: none reported
- external_refs: WorldQuant Alpha101 implementation is cited in alphas101.py; full bibliographic details are not specified in source.
- notes: Formula collection, not a single complete strategy. The backtest example shown uses Alpha083, not every library factor.
- sources: [popbo/alphas — Alpha001–Alpha101 implementation and shared rolling operators](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas101.py#L194-L752); [popbo/alphas — Backtrader factor-quantile portfolio example, code cells 5 and 8](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/2.backtrader.ipynb); [popbo/alphas — Forward-return quantile analysis](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/analy_alphas.py#L9-L72)

### G119 | Alpha101-009 Five-Day Conditional Reversal
- priority: Low — The formula is a transparent daily swing signal, but no factor-specific portfolio or cost-aware out-of-sample result is supplied.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Alpha101 formula inputs: daily close, volume, and/or OHLC
- Overview: A short-horizon reversal score that switches to continuation during a five-session monotonic run. The conditional reversal/continuation behavior is explicit, but the library does not specify a standalone holding rule.
- Stock or ETF Criteria: CN equity assets with the factor's required daily fields.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use the one-day close change with its sign reversed, except when every one-day change over the last five sessions is positive or every change is negative; in either monotonic case use the un-reversed change.
- Exit conditions: not specified in source for the individual factor
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; exact holding period is not specified in source.
- reported_metrics: none reported
- notes: Selected as a standout because its code expresses a clear 1–5 session price/volume swing effect; the repository's example portfolio evaluates Alpha083 instead.
- sources: [popbo/alphas — Alpha101-009 Five-Day Conditional Reversal formula](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas101.py#L248-L254); [popbo/alphas — The repository's factor portfolio example; not a factor-specific exit rule, code cells 5 and 8](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/2.backtrader.ipynb)

### G120 | Alpha101-012 Volume-Confirmed One-Day Reversal
- priority: Low — The formula is a transparent daily swing signal, but no factor-specific portfolio or cost-aware out-of-sample result is supplied.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Alpha101 formula inputs: daily close, volume, and/or OHLC
- Overview: A one-day price-reversal factor gated by the sign of the one-day volume change. It targets short-term price dislocations but the repository does not prescribe a standalone trade horizon.
- Stock or ETF Criteria: CN equity assets with the factor's required daily fields.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal = sign(volume(t)-volume(t-1)) * -(close(t)-close(t-1)); a positive score corresponds to higher volume with a falling close or lower volume with a rising close.
- Exit conditions: not specified in source for the individual factor
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; exact holding period is not specified in source.
- reported_metrics: none reported
- notes: Selected as a standout because its code expresses a clear 1–5 session price/volume swing effect; the repository's example portfolio evaluates Alpha083 instead.
- sources: [popbo/alphas — Alpha101-012 Volume-Confirmed One-Day Reversal formula](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas101.py#L270-L271); [popbo/alphas — The repository's factor portfolio example; not a factor-specific exit rule, code cells 5 and 8](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/2.backtrader.ipynb)

### G121 | Alpha101-101 Intraday Candle-Location Factor
- priority: Low — The formula is a transparent daily swing signal, but no factor-specific portfolio or cost-aware out-of-sample result is supplied.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Alpha101 formula inputs: daily close, volume, and/or OHLC
- Overview: A normalized candle-body direction score uses the day's close relative to its open and range. It is a daily factor input rather than a complete trade rule.
- Stock or ETF Criteria: CN equity assets with the factor's required daily fields.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal = (close-open) / ((high-low)+0.001).
- Exit conditions: not specified in source for the individual factor
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; exact holding period is not specified in source.
- reported_metrics: none reported
- notes: Selected as a standout because its code expresses a clear 1–5 session price/volume swing effect; the repository's example portfolio evaluates Alpha083 instead.
- sources: [popbo/alphas — Alpha101-101 Intraday Candle-Location Factor formula](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas101.py#L725-L726); [popbo/alphas — The repository's factor portfolio example; not a factor-specific exit rule, code cells 5 and 8](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/2.backtrader.ipynb)

### G122 | Alpha191 Formulaic Alpha Library
- priority: Low — Formula definitions are precise but the repository does not provide a verified, cost-aware out-of-sample portfolio test for the full library.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Daily OHLCV, VWAP, rolling ranks, correlations, and 191 formula operators
- Overview: A second formula-alpha collection builds 191 daily cross-sectional signals from price, volume, and market microstructure features. It is a factor research library rather than an end-to-end entry/exit system.
- Stock or ETF Criteria: Universe and required price/volume fields vary by formula; examples use CN equity data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Calculate each Alpha191 factor by asset and date; the code does not specify a universal trading threshold or portfolio selection rule.
- Exit conditions: not specified in source
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; formula-specific holding periods are not specified in source.
- reported_metrics: none reported
- external_refs: Alpha191 formulas are attributed in source to an external formula library; complete bibliographic details are not specified.
- notes: Grouped library entry; no individual-factor rules or exits are asserted here.
- sources: [popbo/alphas — Alpha001–Alpha191 formula definitions](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas191.py#L1-L1231); [popbo/alphas — Forward-return quantile analysis used by the repository](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/analy_alphas.py#L9-L72)

### G123 | Alpha191-020 Six-Session Price Momentum
- priority: Low — The factor has clear six-session momentum logic but no factor-specific cost-aware portfolio test.
- direction: long/short | market: CN A-shares | timeframe: 1d (daily-native)
- indicators: Daily close
- Overview: A simple six-session percentage price change is retained as an Alpha191 factor. It gives an interpretable short-swing momentum feature, but is not independently traded by the repository's example portfolio.
- Stock or ETF Criteria: CN equity assets with six prior daily closes.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal = 100 * (close(t)-close(t-6))/close(t-6); portfolio entry threshold is not specified in source.
- Exit conditions: not specified in source
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars are native; holding period is not specified in source.
- reported_metrics: none reported
- notes: Selected as a standout individual Alpha191 factor because the source rule is directly interpretable as daily swing momentum.
- sources: [popbo/alphas — Alpha020 six-session return formula](https://github.com/popbo/alphas/blob/620594add1c48a4c8b8eb0dca22c2fef9cabccf9/alphas191.py#L251-L253)


## Price Action, Candlestick and Smart-Money Patterns

### G124 | Bollinger-Band Touch then Pullback-to-MA Continuation Entry
- priority: Medium — Crossref to S050 (first pullback after breakout) - the BB-touch-as-breakout definition plus ATR-stop box is a clean variant.
- overlap: S050 (existing catalog) — Uses BB band touch as the 'breakout' trigger (vs price-level breakout in S050); configurable 'Fade the BB Touch' inversion option; touch validity window in bars; ATR box with skew for R:R.
- direction: long/short | market: FX (MT4) | timeframe: chart TF (param) (intraday-adaptable)
- indicators: Bollinger bands (param period/std), MA (param period/method), ATR channel for stop
- Overview: 'Maple Mike' system: a Bollinger-band touch marks the directional impulse; wait for price to pull back to the moving average; enter anticipating continuation back toward the touched band. Stop controlled by ATR-defined range (with vertical skew for R:R control). An explicit band-touch + pullback continuation - the mirror image of band fading and essentially the catalog's first-pullback-after-breakout pattern.
- reported_metrics: none reported
- sources: [matthewkastor/Metatrader — criteria + settings](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/BollingerBandPullbackTrader/README.md); [matthewkastor/Metatrader — EA entry](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/BollingerBandPullbackTrader/BollingerBandPullbackTrader.mq4)

### G125 | ICT Smart-Money-Concepts Signal Primitives (FVG, Order Blocks, BOS/CHoCH, Liquidity, Premium/Discount)
- priority: Medium — Distinctive pattern vocabulary with a real (if contested) thesis - trapped-trader/institutional-rebalancing logic; worth one disciplined daily-bar test of FVG/OB retest.
- direction: long/short | market: any OHLCV | timeframe: any (intraday-native concepts) (intraday-adaptable)
- indicators: Fair Value Gaps, order blocks, swing highs/lows, break of structure, change of character, liquidity sweeps, premium/discount zones, previous high/low
- Overview: The reference open-source implementation of ICT (Inner Circle Trader) smart-money concepts as composable primitives. The canonical strategy built from them: after a break of structure, wait for price to retrace into the originating order block or fair value gap (institutional 'rebalancing' zones) and enter in the BOS direction with a stop beyond the block; liquidity-sweep reversals fade stop-hunts beyond equal highs/lows. Popular discretionary framework with unclear quantified edge - worth a disciplined daily-bar test.
- Stock or ETF Criteria: Liquid instruments with clean OHLC structure.
- Stocks to avoid and known failure conditions: SMC concepts are subjective - the library fixes definitions but signal quality is regime-dependent; retail-crowded concepts may be arbitraged away.
- Entry conditions: Canonical: bullish BOS -> mark order block/FVG -> buy limit on retrace into zone. Bearish mirror after bearish BOS. Liquidity sweep variant: fade the sweep of equal highs/lows back inside range.
- Exit conditions: Target opposing liquidity pool (recent swing high/low) or next FVG; stop beyond the order block extreme.
- Trade management: Fixed-fractional risk per zone trade; multiple zones can pyramid.
- Target: Opposite liquidity / 2-3R typical.
- Risk factors and risk management: Zone invalidation = stop beyond block; premium/discount zone filter to avoid mid-range entries.
- Best known candlestick timeframe(s): 1h-4h native in ICT practice; daily-bar FVG/OB retests are a legitimate adaptation (fewer, wider zones).
- reported_metrics: none reported
- notes: Library has no executable strategy - record describes the canonical usage; STAR-EA repo (elsewhere in batch) is a 12-scenario implementation to compare.
- sources: [joshyattridge/smart-money-concepts — fvg/order blocks/BOS/CHoCH/liquidity/swing/premium-discount functions](https://github.com/joshyattridge/smart-money-concepts/blob/1b62fd6c41e1f508e7ed76831a039fa4c82d42f6/smartmoneyconcepts/smc.py); [joshyattridge/smart-money-concepts — indicator definitions](https://github.com/joshyattridge/smart-money-concepts/blob/1b62fd6c41e1f508e7ed76831a039fa4c82d42f6/README.md)

### G126 | STAR-EA ICT 12-Scenario Engine with AMD Phase Gating
- priority: Medium — The scenario-classifier-gates-technique architecture is the transferable idea (regime-gated pattern selection) even if ICT edge itself is unproven.
- direction: long/short | market: FX (MT5) | timeframe: intraday (intraday-adaptable)
- indicators: Order blocks, FVG, OTE, liquidity sweeps, CRT, Judas swing, Silver Bullet, AMD (Accumulation-Manipulation-Distribution) phase detector, 12-scenario market classifier
- Overview: A comprehensive ICT-concepts EA: a 12-scenario market-classification engine gates which technique is allowed (breaker/Judas/CRT/Silver-Bullet mapped to BOS/LIQ/TC/FVG scenario flags), plus an AMD phase detector that suppresses entries in accumulation, trades the manipulation fake-out (Judas swing), and follows distribution. The most complete codification of smart-money-concept filtering in the inventory - the value is the scenario-GATE: only trade the pattern that matches the current market state.
- Stock or ETF Criteria: FX majors on MT5; intraday sessions (London/NY killzones implied by SB).
- Stocks to avoid and known failure conditions: Massive parameter surface = overfit risk; ICT concepts lack peer-reviewed evidence; intraday session logic doesn't transfer to daily bars 1:1.
- Entry conditions: Scenario-gated: entry allowed only when the market classifier's scenario flag matches the technique (e.g., Judas swing needs allowLIQ; Silver Bullet needs allowTC or allowFVG). AMD accumulation blocks entries unless AMD_TradeInManip.
- Exit conditions: Per-technique ICT targets (opposing liquidity / structure); distribution timeout 48 bars.
- Trade management: 12-scenario gate as the position-selection layer; per-technique params.
- Target: Technique-specific ICT objectives.
- Risk factors and risk management: Scenario gating reduces false entries; AMD phase detector adds session context.
- Best known candlestick timeframe(s): Intraday (killzone/session dependent); daily adaptation possible for OB/FVG retest components only.
- reported_metrics: none reported
- notes: Fix-log comments in source document the gating bugs found - useful as a what-not-to-do reference.
- sources: [NadirAliOfficial/STAR-EA-v11.20 — scenario gate mapping + DetectAMDPhase + technique flags](https://github.com/NadirAliOfficial/STAR-EA-v11.20/blob/32c601e90bebcfc141439cca294533046c1330f2/STAR_v11.20.mq5); [NadirAliOfficial/STAR-EA-v11.20 — 12-scenario overview](https://github.com/NadirAliOfficial/STAR-EA-v11.20/blob/32c601e90bebcfc141439cca294533046c1330f2/README.md)

### G127 | Bollinger Bands Bottom-W Pattern Reversal
- priority: Low — The rule is distinctive and explicit, but its absolute tolerances are scale-sensitive and no out-of-sample results are shown.
- direction: long | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: 20-session SMA, ±2 standard-deviation bands, five-node W pattern
- Overview: A bottom-W pattern is detected from two lower-band tests and intervening midpoint recovery, with a breakout above the upper band confirming the pattern. The code treats bandwidth contraction as a condition to clear the held position.
- Stock or ETF Criteria: Daily equity prices with at least 75 sessions of history.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Use Bollinger(20,2), a 75-session pattern window, alpha=0.0001 band-distance tolerance, and beta=0.0001 bandwidth contraction threshold. After a prior lower-band touch, a midpoint recovery and a higher second low, enter long when price crosses above the upper band.
- Exit conditions: Exit when band standard deviation falls below beta, provided no new pattern signal is being generated; no stop or time stop is specified.
- Trade management: One long position at a time; code uses cumulative signals to track holding state.
- Target: Upper-band breakout confirms entry; no fixed profit target.
- Risk factors and risk management: Pattern thresholds are hardcoded extremely tightly and likely depend on price scale; no trading costs are applied.
- Best known candlestick timeframe(s): Daily bars are native.
- reported_metrics: none reported
- notes: The code detects a W-shaped bottom, not a generic lower-band buy.
- sources: [je-suis-tm/quant-trading — Bands and five-node W detection, tolerances, and contraction exit](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Bollinger%20Bands%20Pattern%20Recognition%20backtest.py#L39-L169)

### G128 | Heikin-Ashi Consecutive-Candle Trend Reversal
- priority: Low — The code is explicit but its candle direction is counterintuitive and the backtest does not model costs.
- direction: long | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: Heikin-Ashi OHLC and consecutive candle-body comparisons
- Overview: The strategy transforms OHLC into Heikin-Ashi candles and uses a multi-candle shape to open longs, with a mirrored opposite-color setup to close accumulated long exposure. It is a price-action pattern rule rather than an oscillator.
- Stock or ETF Criteria: Daily OHLC equities.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute HA close=(O+H+L+C)/4 and HA open=(prior HA open+prior HA close)/2. Long when HA open>HA close, HA open equals HA high, current body exceeds prior body, and the prior HA candle is bearish; cap accumulated longs at the configured `stls` value.
- Exit conditions: On a bullish HA candle with HA open equal to HA low and a preceding bullish candle, close the accumulated long position; no short position is opened by the code.
- Trade management: Cumulative long positions are capped by the `stls` parameter; no stop/target values are specified.
- Target: Exit on the opposite Heikin-Ashi candle condition.
- Risk factors and risk management: The source's long trigger uses a bearish candle shape; test semantics carefully. No transaction costs are modeled.
- Best known candlestick timeframe(s): Daily bars are used in the example.
- reported_metrics: none reported
- notes: The comments and signal labels should be verified against the actual intended Heikin-Ashi convention.
- sources: [je-suis-tm/quant-trading — Heikin-Ashi transformation and signal/position logic](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Heikin-Ashi%20backtest.py#L41-L123)

### G129 | RSI Head-and-Shoulders Pattern Reversal
- priority: Low — The pattern has explicit exits but is a small-sample heuristic without cost or OOS evidence.
- direction: short | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: RSI(14) extrema forming a head-and-shoulders pattern
- Overview: The pattern module searches RSI for a head-and-shoulders configuration and shorts on the completed pattern. It exits when RSI rises four points from the entry reading or after five sessions.
- Stock or ETF Criteria: Daily equities with 14-session RSI history.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: On a 14-session RSI series, identify the notebook's head-and-shoulders extrema and require the terminal price/pattern constraints; the node-difference tolerance is configurable in the code.
- Exit conditions: Close after RSI rises more than 4 points from entry or after more than 5 sessions.
- Trade management: One short pattern position at a time; exact sizing is not specified.
- Target: No profit target; RSI +4 or five-session time exit.
- Risk factors and risk management: Pattern tolerances are hardcoded/tuned by inspection and no transaction costs are modeled.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- notes: The author comments that the simple RSI extreme strategy is not expected to be profitable; do not treat that claim as validated evidence.
- sources: [je-suis-tm/quant-trading — Pattern node logic and RSI/time exits](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/RSI%20Pattern%20Recognition%20backtest.py#L151-L305)

### G130 | Shooting-Star Candlestick Reversal
- priority: Low — The candle definition is explicit but is a heuristic with a very short sample and no realistic costs.
- direction: short | market: Yahoo Finance equities | timeframe: 1d (daily-native)
- indicators: Eight OHLC shooting-star criteria, body/range ratios
- Overview: The strategy shorts a shooting-star candle defined by eight price-action criteria, then exits on a 5% absolute move or a seven-session time stop. It is a reversal pattern that assumes rejection of higher prices.
- Stock or ETF Criteria: Daily OHLC equity bars.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Require all eight candle conditions using `lower_bound=0.2` and `body_size=0.5`; enter short at the signal close.
- Exit conditions: Exit when absolute close-to-entry change exceeds 5% (either favorable or adverse) or after 7 sessions.
- Trade management: One short per signal; `stop_threshold=0.05` and `holding_period=7`.
- Target: Symmetric ±5% close-to-entry exit or seven-session time stop.
- Risk factors and risk management: The absolute-value exit closes both profitable and losing 5% moves; no costs or regime filter are present.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- notes: The 5% trigger is symmetric; it functions as both a profit-taking and stop exit.
- sources: [je-suis-tm/quant-trading — Shooting-star criteria and short signal/stop/time exit](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Shooting%20Star%20backtest.py#L26-L120)


## Volatility and Options-Derived Signals

### G131 | Long/Short Straddle Volatility Trading (mcf-long-short IBKR)
- priority: Medium — Canonical vol-event strategy; long/short straddle is the standard way to trade expected-move mispricing.
- direction: long/short | market: equity options via IBKR | timeframe: event/straddle holding (daily-native)
- indicators: Straddle construction on underlying vol expectations (Yahoo data + TWS)
- Overview: Trades long and short straddles around expected volatility events: long straddles before expected big moves (earnings/uncertainty), short straddles into overpriced vol. Vol-expectation-mispricing play.
- Stock or ETF Criteria: Optionable names with catalyst/vol edge.
- Stocks to avoid and known failure conditions: Short straddle = unlimited loss; long straddle bleeds theta if the move underwhelms.
- Entry conditions: Long straddle into expected vol expansion; short straddle when IV appears rich vs expected move.
- Exit conditions: Post-move unwind / expiry approach.
- Trade management: Leg-based option positions.
- Target: Vol-move capture.
- Risk factors and risk management: IV crush on longs; tail risk on shorts.
- Best known candlestick timeframe(s): Daily event windows.
- reported_metrics: none reported
- notes: Overlap S020/S054 earnings-adjacent family; options version.
- sources: [mcf-long-short/ibkr-options-volatility-trading — straddle impl](https://github.com/mcf-long-short/ibkr-options-volatility-trading/blob/f63c03b74439488d723fecdbc7fd3edd51f7bbbd/README.md)

### G132 | Options Wheel (CSP -> Covered Call) - alpacahq template
- priority: Medium — Well-defined premium strategy with a genuine risk-premium thesis (vol premium), slow cadence.
- direction: long | market: US equities options | timeframe: weekly-ish option cycles (daily-native)
- indicators: Cash-secured put at support/oversold -> assigned shares -> covered calls
- Overview: Runnable wheel template: sell CSPs on names you're willing to own, collect premium or take assignment, then sell covered calls until called away. Harvests option premium + mean reversion on quality names; a slow but well-defined income strategy.
- Stock or ETF Criteria: Quality/liquid names you'd accept ownership of; adequate put/call liquidity.
- Stocks to avoid and known failure conditions: Assignment in a real downtrend (bag-holding); premium is thin in low-IV regimes.
- Entry conditions: Sell OTM CSP (delta/days per template).
- Exit conditions: Assigned -> covered calls; repeat.
- Trade management: Wheel state machine per symbol.
- Target: Premium capture + assignment exit at strike.
- Risk factors and risk management: Downside equity risk on assignment.
- Best known candlestick timeframe(s): Daily bars for state; weekly-ish expiries.
- reported_metrics: none reported
- notes: Slow strategy; pairs with S013/S014 reversion universe selection.
- sources: [alpacahq/options-wheel — wheel template](https://github.com/alpacahq/options-wheel/blob/3698429289065ceb0c13ffcdc31a966c576779ad/README.md)

### G133 | Short-Term-Reversal Option Setups with Staged Entry (randomwalkhan)
- priority: Medium — Staged-entry protocol is a practical twist on the reversal family worth porting to equity entries too.
- direction: long | market: US equities via option overlays | timeframe: 1d (daily-native)
- indicators: Short-term reversal signal on the underlying (multi-day down moves) driving option entries; staged-entry backtesting + live paper trading
- Overview: Researches short-term-reversal setups traded through options: identifies oversold underlying conditions, then enters option structures in stages rather than all at once, with a live paper-trading validation path. The transferable part is the staged-entry protocol on a daily reversal signal - splitting entries across days to smooth fill/timing luck.
- Stock or ETF Criteria: Optionable US equities with reversal setups.
- Stocks to avoid and known failure conditions: Options overlay adds spread+decay drag on top of the reversal signal; staged entry delays a fast-bounce.
- Entry conditions: Underlying oversold/reversal trigger -> staged option entries (multiple partial entries across sessions).
- Exit conditions: Signal reversion / option-rule exits.
- Trade management: Stage sizes; paper-trade tracked.
- Target: Reversal capture via options.
- Risk factors and risk management: Theta decay + reversal failing (continuation).
- Best known candlestick timeframe(s): Daily bars (entry signal), option execution.
- reported_metrics: none reported
- notes: Overlap S027-S031 reversal family; the options+staging wrapper is the distinctive part.
- sources: [randomwalkhan/Short-Term-Reversal-Strategy — reversal + staged entry](https://github.com/randomwalkhan/Short-Term-Reversal-Strategy/blob/734f750d0538c2cc00531fb2d50534f900b2a8e8/README.md)

### G134 | Systematic Equity-Options Volatility Research (anthonymakarewicz)
- priority: Medium — Vol-signal research layer - complements the term-structure family (S062).
- direction: long/short | market: equity options | timeframe: 1d (daily-native)
- indicators: Systematic vol signals (IV surface / variance research)
- Overview: Systematic volatility research and backtesting for equity options - the useful extraction is its vol-signal research (term structure, IV-vs-realized, vol-of-vol) applied to option trades.
- Stock or ETF Criteria: Equity options chains.
- Stocks to avoid and known failure conditions: Vol research needs clean option data.
- Entry conditions: Vol-signal conditioned option positions.
- Exit conditions: Per strategy.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Overlap S061/S062 vol family.
- sources: [anthonymakarewicz/volatility-trading — vol research (+2 more verified paths)](https://github.com/anthonymakarewicz/volatility-trading/blob/a866b6eee79d9a0c0bf81a1b82208dc6da6aae66/README.md); [anthonymakarewicz/volatility-trading — notebooks](https://github.com/anthonymakarewicz/volatility-trading/blob/a866b6eee79d9a0c0bf81a1b82208dc6da6aae66/notebooks); [anthonymakarewicz/volatility-trading — src](https://github.com/anthonymakarewicz/volatility-trading/blob/a866b6eee79d9a0c0bf81a1b82208dc6da6aae66/src)

### G135 | AI-Trader NSE Options - Dual ML + Institutional Flow
- priority: Low — Intraday + flow-data dependency; flow-condition idea notable.
- direction: long/short | market: NSE F&O options | timeframe: intraday (intraday-only)
- indicators: Dual ML models + institutional flow analysis + regime-adaptive option strategies
- Overview: Intraday options system for NSE combining two ML models with institutional-flow signals and regime-adaptive strategy selection - a flow-informed intraday option trader.
- Stock or ETF Criteria: NSE F&O.
- Stocks to avoid and known failure conditions: Intraday options; flow data hard to source.
- Entry conditions: ML + flow confluence signals.
- Exit conditions: Intraday rules.
- Trade management: Regime-adaptive sizing.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday.
- reported_metrics: none reported
- notes: Intraday-only.
- sources: [aaryansinha16/AI-trader — dual ML + flow (+1 more verified paths)](https://github.com/aaryansinha16/AI-trader/blob/de830f95c462c8142acb8191c62d3404868484aa/README.md); [aaryansinha16/AI-trader — models](https://github.com/aaryansinha16/AI-trader/blob/de830f95c462c8142acb8191c62d3404868484aa/models)

### G136 | Gamma Scalping Delta-Hedge Template (alpacahq)
- priority: Low — Real vol edge but cost-fragile at retail; document for vol family.
- direction: market-neutral | market: equity options | timeframe: intraday-daily hedge (intraday-adaptable)
- indicators: Long option gamma + delta-hedging the underlying
- Overview: Long a straddle/option then delta-hedge the underlying repeatedly to monetize realized volatility - profit if realized vol exceeds implied at purchase. Classic realized-vs-implied vol edge, but needs cheap hedge execution.
- Stock or ETF Criteria: Liquid option + underlying.
- Stocks to avoid and known failure conditions: Realized < implied = bleed; hedge costs kill the edge at retail spreads.
- Entry conditions: Buy option/straddle when IV looks cheap vs expected realized.
- Exit conditions: Continuous delta hedges until expiry/closeout.
- Trade management: Rehedge on delta drift thresholds.
- Target: Realized > implied spread capture.
- Risk factors and risk management: Hedge execution cost.
- Best known candlestick timeframe(s): Hedge loop intraday; position days-weeks.
- reported_metrics: none reported
- notes: Execution-sensitive.
- sources: [alpacahq/gamma-scalping — gamma scalp (+1 more verified paths)](https://github.com/alpacahq/gamma-scalping/blob/c80b5e36f4b375b2c096d1370f6d5962f5abb630/main.py); [alpacahq/gamma-scalping — README.md](https://github.com/alpacahq/gamma-scalping/blob/c80b5e36f4b375b2c096d1370f6d5962f5abb630/README.md)

### G137 | Heikin-Ashi-Style Option Long Straddle Entry
- priority: Low — The option structure is explicit but the trigger is not an economic volatility signal and exit rules are absent.
- direction: long | market: EURO STOXX 50 index options | timeframe: Option-expiry event; underlying bars not specified (4h-plus)
- indicators: Call/put premium difference for matched strike and expiry
- Overview: The notebook tests a long straddle when call and put premiums are close in price, seeking a large move in either direction. It is not a volatility forecast; the signal is based on premium symmetry.
- Stock or ETF Criteria: Matched call/put strike and expiry; example data are EURO STOXX 50 options.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Buy a call and put with the same strike and expiry when |call premium−put premium|<threshold; the threshold is an input and not fixed in source.
- Exit conditions: not specified in source
- Trade management: Long both option legs; sizing, expiry management, and transaction costs are not specified.
- Target: Profits from a sufficiently large move in either direction; no numeric target is supplied.
- Risk factors and risk management: Long premium can decay to zero; code does not model Greeks, volatility surface, or exit timing.
- Best known candlestick timeframe(s): Options expiry/volatility event strategy; daily-bar mapping is not specified.
- reported_metrics: none reported
- notes: The source code demonstrates only a premium-symmetry entry flag, not a complete backtest.
- sources: [je-suis-tm/quant-trading — Long-straddle premise and premium-difference trigger](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/Options%20Straddle%20backtest.py#L20-L141)

### G138 | Options Strategy Backtest Cookbook Set (PyPatel + Option Greeks book)
- priority: Low — Option-structure taxonomy; the underlying signals are standard.
- direction: long/short | market: equity/index options | timeframe: 1d (daily-native)
- indicators: Technical indicators -> option-structure selection (directional spreads, condors, straddles) + Greeks-based management
- Overview: Two related collections of option strategy implementations driven by technical indicators and quantitative methods (PyPatel) and the 'Option Greeks Strategies & Backtesting in Python' book code - a taxonomy of directional/vol option plays with Greeks management.
- Stock or ETF Criteria: Optionable names; Greeks data.
- Stocks to avoid and known failure conditions: Option overlays inherit signal fragility + add Greeks exposure.
- Entry conditions: Indicator signal -> mapped option structure.
- Exit conditions: Greeks/underlying-signal exits.
- Trade management: Spread/condor/straddle constructions.
- Target: Vol/direction premium.
- Risk factors and risk management: Structure-specific Greeks.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Grouped options-strategy set.
- sources: [PyPatel/Options-Trading-Strategies-in-Python — option strategies](https://github.com/PyPatel/Options-Trading-Strategies-in-Python/blob/c7b8a0de9b7232a1615a02acbed70bf2959fbe44/README.md); [OptionsnPython/Option-strategies-backtesting-in-Python — book code](https://github.com/OptionsnPython/Option-strategies-backtesting-in-Python/blob/275deeff8b55ae1c0d759ee341cb2c7b0f32f4c6/README.md)

### G139 | VIX Term-Structure and Pair-Spread Volatility Study
- priority: Low — Research scaffolding without executable entry/exit; useful mainly as a VIX term-structure data pipeline.
- direction: market-neutral | market: equity pairs + VIX futures | timeframe: 1d (daily-native)
- indicators: VIX futures term structure (settlement vs expiry), pair spread volatility
- Overview: Research notebooks that crawl CBOE VIX futures settlement data and 30 equity/ETF spot+option prices to map the volatility term structure and build a pairs-trading strategy conditioned on vol regime. The deliverable is research groundwork for a vol-regime-filtered pair trade rather than a complete executable strategy.
- Stock or ETF Criteria: 30 equity/ETF names with both spot and CBOE option chains; VIX futures settlement series.
- Stocks to avoid and known failure conditions: not specified in source. Suggested: options data gaps in the crawl can bias vol estimates; VIX futures series has missing '-' settlements requiring cleaning.
- Entry conditions: not specified in source. Strategy conditions pair-spread trades on VIX term-structure state; exact trigger not specified in source. Suggested: long pair spread when term structure flips to backwardation.
- Exit conditions: not specified in source.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: term-structure timing is regime-dependent; backwardation episodes are brief.
- Best known candlestick timeframe(s): Daily bars; weekly VIX futures settlement blocks.
- reported_metrics: none reported
- notes: Value is the CBOE data pipeline + vol term-structure features, not a finished strategy.
- sources: [linhnguyen215538/Volatility-Study — VIX term-structure construction](https://github.com/linhnguyen215538/Volatility-Study/blob/20b73c69e438e83e556ed9cb1d4e1c6741489b7a/doc/Vol%20Study%20-%20Part%203%20%2B%204.ipynb); [linhnguyen215538/Volatility-Study — spot/options data collection](https://github.com/linhnguyen215538/Volatility-Study/blob/20b73c69e438e83e556ed9cb1d4e1c6741489b7a/doc/Vol%20Study%20-%20Part%201.ipynb)


## Calendar, Session and Overnight Effects

### G140 | Equity Overnight Return Hold
- priority: Medium — 
- overlap: S021 (existing catalog) — The notebook measures close-to-open returns across its equity universe with daily rebalance; it does not add a stock-selection signal beyond the overnight holding window.
- direction:  | market: US equities | timeframe: Close-to-open (intraday-adaptable)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [rolling-panda-san/notebooks — Separates daily close-to-open from open-to-close returns; code cell 4](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/overnight_returns.ipynb)

### G141 | Close-to-Open Gap Reversal Forecast
- priority: Low — Overnight gaps are a plausible underreaction effect, but the code depends on intraday execution and beta/industry data.
- direction: long/short | market: US equities | timeframe: Daily close-to-open plus intraday bars (intraday-adaptable)
- indicators: Overnight log return, beta adjustment, industry demeaning, 2% absolute-gap filter
- Overview: The strategy fades sufficiently large overnight gaps after removing the market component. Six time-of-day coefficients allow the expected reversal to vary as the session progresses.
- Stock or ETF Criteria: Expandable/liquid US equities with daily and 15-minute or 30-minute OHLC, beta and industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute log(open/previous close), beta-adjust and industry-demean; zero signals for absolute gaps below 2%; combine current gap with lagged daily gaps and regression coefficients.
- Exit conditions: Alpha is used for same-session intraday forecasts; explicit stop, profit target and holding time are not specified.
- Trade management: Signal forecasts are consumed by the order simulator; there is no direct order-sizing logic in c2o.py.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Requires the overnight close-to-open window and intraday bars; a daily close-to-close proxy would not be equivalent.
- reported_metrics: none reported
- notes: The module generates an alpha forecast, not a complete standalone trade; no execution costs are specified here.
- sources: [noterminusgit/statarb — Gap calculation, 2% filter, daily/intraday fitting and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/c2o.py#L104-L335)

### G142 | Gao 2018 Equity ETF Intraday Momentum
- priority: Low — The session-momentum thesis is plausible but the signal is intraday-only and its implementation is private.
- direction: long | market: US equity ETFs | timeframe: Intraday bars; 09:30–16:00 session (intraday-only)
- indicators: First-hour return r1 and prior-session return r12
- Overview: The notebook compares first-hour and prior-session return signals for an intraday ETF portfolio. The cited approach tests whether early-session price discovery predicts late-session returns.
- Stock or ETF Criteria: Equity ETFs with intraday bars.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Run the private `IntradayMomentumGao2018` signal with r1-only, r12-only, and combined r1/r12 variants; exact thresholds/position mapping are not exposed in the notebook.
- Exit conditions: Exit at the session close; the backtest uses close-based PnL for the 09:30–16:00 session.
- Trade management: Daily close liquidation; sizing and costs are delegated to private `vivace`.
- Target: Close at 16:00; no profit target.
- Risk factors and risk management: Intraday spreads, opening volatility and execution slippage can dominate the signal; no net cost model is visible.
- Best known candlestick timeframe(s): Intraday-only; daily OHLC cannot reproduce the signal.
- reported_metrics: none reported
- external_refs: Gao et al. (2018), as named by the notebook; complete citation is in the reference cell.
- notes: The notebook compares r1, r12 and combined signals to a long-only baseline.
- sources: [rolling-panda-san/notebooks — r1/r12 signal variants, intraday ETF universe and session PnL; code cells 4, 30, 39, 45, and 54](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/equity_etf_intraday_momentum.ipynb)

### G143 | Hour-of-Day Session Gating (HourBasedStrategy)
- priority: Low — Session-timing overlay; concept useful for the overnight-hold entry-timing question.
- direction: long | market: crypto (freqtrade) | timeframe: 1h (intraday-adaptable)
- indicators: UTC hour-of-day bucket + base entry conditions
- Overview: Gates entries on the UTC hour-of-day - trades only in historically-favorable hourly windows. Documents the sessionality idea cleanly: crypto has repeatable intraday liquidity/vol cycles; equities analog = time-of-day entry filters (which the daily-bar catalog can't directly use but informs overnight-hold timing).
- Stock or ETF Criteria: Crypto pairs; needs hour-of-day edge map.
- Stocks to avoid and known failure conditions: Hour buckets overfit to the sample window.
- Entry conditions: Entry condition AND hour_of_day in allowed set.
- Exit conditions: ROI/stoploss.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Small-sample calendar effects.
- Best known candlestick timeframe(s): 1h; equity analog = intraday entry-timing overlays.
- reported_metrics: none reported
- notes: Overlap S079-S082 calendar family conceptually.
- sources: [freqtrade/freqtrade-strategies — hour gating](https://github.com/freqtrade/freqtrade-strategies/blob/f3340ce11f5bdf62f598522e64d1f5638eaa13f5/user_data/strategies/HourBasedStrategy.py)

### G144 | London Session Opening-Range Breakout
- priority: Low — The session effect is clear but highly execution- and timezone-dependent.
- direction: long/short | market: FX currency pairs | timeframe: Intraday, London open (intraday-only)
- indicators: Pre-London hour high/low, first 30-minute breakout
- Overview: This session strategy measures the hour before London opens, trades a breakout during the first 30 minutes, and exits on target, stop, or the end of the trading window. The thesis is concentrated FX liquidity during the London session.
- Stock or ETF Criteria: FX pairs with intraday bars around London open.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Build upper/lower thresholds from the last pre-London hour's high/low; during the first 30 minutes after London open, enter long/short when price breaks the corresponding threshold. The exact threshold multiplier is configurable and not specified in source.
- Exit conditions: Exit at configured stop loss/profit target or by 12:00 London/ET session end if neither is hit.
- Trade management: At most one open session position; configured profit target and stop, followed by forced session close.
- Target: Configured stop/target; otherwise flatten at session end.
- Risk factors and risk management: Session boundaries and daylight-saving handling are material; no realistic transaction-cost model is shown.
- Best known candlestick timeframe(s): Intraday only.
- reported_metrics: none reported
- notes: The notebook uses hardcoded market hours and does not establish out-of-sample robustness.
- sources: [je-suis-tm/quant-trading — Session rationale and threshold/signal framework](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/London%20Breakout%20backtest.py#L9-L25); [je-suis-tm/quant-trading — Signal and session exit implementation](https://github.com/je-suis-tm/quant-trading/blob/611b73f2c3f577ac5b28aaa19ac8c43d3236c7a5/London%20Breakout%20backtest.py#L51-L180)

### G145 | Month-End Long After Negative Prior Month
- priority: Low — A calendar reversal thesis is explicit, but timing internals and costs are unavailable.
- direction: long | market: US equities | timeframe: Daily bars; month-end schedule (daily-native)
- indicators: Month-end timing and prior-month return sign
- Overview: A calendar strategy enters near month-end only after a negative prior-month return, testing a conditional month-end reversal effect. It is distinct from an unconditional turn-of-month premium.
- Stock or ETF Criteria: US equity instruments in the notebook.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Call `MonthEndLong(n_before=5, n_after=0, is_prev_neg=True)` with a 200-session trend filter; exact calendar-date mapping is inside private `vivace`.
- Exit conditions: Carry the position through month-end; the private signal's exact exit timestamp is not specified in the clone.
- Trade management: The pipeline carries the signal for five sessions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars with calendar-aware session data.
- reported_metrics: none reported
- notes: The notebook is part of its Connors short-term study; no repo-claimed metrics are included here.
- sources: [rolling-panda-san/notebooks — MonthEndLong configuration and 200-day gate; code cell 22](https://github.com/rolling-panda-san/notebooks/blob/ce05661a5d4b3cdb439ece27dcccba88e2cb130a/equity_short_term_trading_connors.ipynb)

### G146 | Weekday-Conditioned Beta-Adjusted Return Forecast
- priority: Low — Calendar conditioning is precisely defined but is an exploratory factor without cost-aware OOS validation.
- direction: long/short | market: US equities | timeframe: Daily (daily-native)
- indicators: Beta-divided daily returns and weekday-specific regression coefficients
- Overview: The strategy fits separate return-predictability coefficients for each weekday, testing whether beta-adjusted return effects vary with the trading calendar.
- Stock or ETF Criteria: US equities with weekday calendar, beta, and daily return data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute o2c0=log_return/predicted_beta, create daily lags, and apply the coefficient set corresponding to Monday–Friday.
- Exit conditions: Forecast positions refresh daily; no explicit price exit is specified.
- Trade management: Daily alpha forecasts feed the optimizer; no separate stop or sizing is specified.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars only.
- reported_metrics: none reported
- notes: The code comments describe weekday coefficient differences as a hypothesis; no economic mechanism is verified.
- sources: [noterminusgit/statarb — Weekday-specific regression of beta-adjusted return lags](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/badj_dow_multi.py#L137-L376)


## News, Sentiment and Alternative Data

### G147 | Binance News-Sentiment Bot - Headline Sentiment -> Buy New Listings/Alts
- priority: Medium — Sentiment-driven entries on a feed universe - the equity adaptation is a real catalog candidate.
- direction: long | market: crypto (Binance) | timeframe: news-cycle (intraday-adaptable)
- indicators: Scraped news-headline sentiment across top ~100 crypto feeds
- Overview: Aggregates sentiment from ~100 crypto news feeds and buys the coins whose headlines score positive - an early retail 'news-sentiment momentum' bot. Thesis: positive headline flow precedes retail-driven pumps in crypto; the equity analog is the catalog's news-sentiment family.
- Stock or ETF Criteria: Top crypto pairs on Binance with feed coverage.
- Stocks to avoid and known failure conditions: Sentiment->price lag and headline spam; sentiment scoring is dictionary-based (pre-LLM).
- Entry conditions: Coin sentiment score > threshold -> buy (per coin aggregation).
- Exit conditions: Sell on sentiment decay / profit target.
- Trade management: Per-coin positions.
- Target: Sentiment-driven move.
- Risk factors and risk management: Headline manipulation; stale feeds.
- Best known candlestick timeframe(s): News-cycle; equity analog = S056-S060 (LLM news sentiment).
- reported_metrics: none reported
- notes: Overlap S056-S060 sentiment family.
- sources: [CyberPunkMetalHead/Binance-News-Sentiment-Bot — feeds + logic](https://github.com/CyberPunkMetalHead/Binance-News-Sentiment-Bot/blob/099b2039d51b155473555c3d104a1770024adb69/README.md); [CyberPunkMetalHead/Binance-News-Sentiment-Bot — sentiment->buy](https://github.com/CyberPunkMetalHead/Binance-News-Sentiment-Bot/blob/099b2039d51b155473555c3d104a1770024adb69/src/main.py)

### G148 | Earnings-Surprise Drift Factor
- priority: Medium — 
- overlap: S054 (existing catalog) — Uses `EPS_diff_mean/EPS_median` and activates when new estimate data arrive (`std_diff>0`); its distributed-lag forecast is an event-data variant of the catalog's SUE/PEAD approach.
- direction:  | market: US equities | timeframe: Daily around earnings events (daily-native)
- indicators: 
- Overview: 
- reported_metrics: none reported
- sources: [noterminusgit/statarb — EPS surprise/consensus ratio, event filter and distributed-lag forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/eps.py#L96-L336); [noterminusgit/statarb — Production earnings-surprise signal generator](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/prod_eps.py)

### G149 | WSB Mention-Frequency Deviation Long/Short
- priority: Medium — Early retail-attention anomaly implementation - the attention-deviation signal is real alpha-candidate material for the sentiment family.
- direction: long/short | market: equities (WSB-mentioned tickers) | timeframe: 1d (daily-native)
- indicators: Daily ratio of ticker mentions to total subreddit activity vs running average
- Overview: Trades tickers whose WallStreetBets mention frequency deviates >25% from the running average mention ratio, with a 3-day delay to avoid contemporaneous noise. Variants test both directions: strategy1 longs on mention spikes and shorts on drops; the author reports whichever variant beat buy-and-hold per ticker. Thesis: retail-attention spikes precede outsized moves (attention-driven volatility, cf. Barber-Odean attention effect).
- Stock or ETF Criteria: WSB-mentioned equities (companylist.csv universe).
- Stocks to avoid and known failure conditions: Attention->direction mapping is ambiguous (the repo tries long AND short variants, i.e., sign is unclear); 3-day delay may miss the move; small historical window (2016-2017 era).
- Entry conditions: |mention_ratio - avg| > 25% of avg -> enter; direction by variant (spike->long in s1, spike->short in s3/s7).
- Exit conditions: Next day's deviation signal; positions tracked per day.
- Trade management: Per-ticker strategy variant selection by backtest.
- Target: Attention-driven move capture.
- Risk factors and risk management: Unstable direction mapping; data-mined variant selection.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- external_refs: Barber & Odean attention-driven buying literature
- sources: [theriley106/TheWSBIndex — strategy1/3/5/6/7 mention-ratio deviation rules](https://github.com/theriley106/TheWSBIndex/blob/01565badfda01174069855d9e369e830773aa656/algo.py); [theriley106/TheWSBIndex — mention vs volume comparison](https://github.com/theriley106/TheWSBIndex/blob/01565badfda01174069855d9e369e830773aa656/compareToVolume.py)

### G150 | 10-K Filing NLP Long-Term Growth Prediction
- priority: Low — Distinctive alt-data (filings NLP) but slow cadence.
- direction: long | market: US equities (SEC 10-K filers) | timeframe: annual/quarterly (daily-native)
- indicators: NLP on 10-K filing text -> long-term growth prediction
- Overview: Parses SEC 10-K filings with NLP to predict long-term stock growth - a slow, fundamentals-NLP alpha. Distinct from headline sentiment: filings carry richer, legally-audited text.
- Stock or ETF Criteria: SEC filers.
- Stocks to avoid and known failure conditions: Annual cadence = few signals; survivorship in the filing set.
- Entry conditions: Long names with positive filing-text growth signal.
- Exit conditions: Hold long horizon.
- Trade management: Portfolio of screened names.
- Target: Long-term growth capture.
- Risk factors and risk management: Slow signal; text-parsing fragility.
- Best known candlestick timeframe(s): Daily evaluation, annual signal.
- reported_metrics: none reported
- notes: Merge with sentiment/alt-data group.
- sources: [deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports — EDGAR + model (+3 more verified paths)](https://github.com/deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports/blob/34b4acff04d913863c13e1cbd452b79b4d2a8215/Python_Code/Model_Create_Train_Test.ipynb); [deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports — Python_Code/Process_Data_EDA.ipynb](https://github.com/deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports/blob/34b4acff04d913863c13e1cbd452b79b4d2a8215/Python_Code/Process_Data_EDA.ipynb); [deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports — Python_Code/WebScrapping.ipynb](https://github.com/deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports/blob/34b4acff04d913863c13e1cbd452b79b4d2a8215/Python_Code/WebScrapping.ipynb)

### G151 | Analyst Price-Target Deviation Reversion
- priority: Low — A plausible analyst-underreaction thesis is present but point-in-time target data and OOS costs are not established.
- direction: long/short | market: US equities | timeframe: Daily forecasts from analyst target updates (daily-native)
- indicators: Log of median analyst target divided by close, industry-demeaned
- Overview: The factor measures the gap between analyst consensus target and current price, then predicts gradual convergence with distributed lags. It is an analyst-valuation signal rather than an earnings-surprise signal.
- Stock or ETF Criteria: US equities with point-in-time analyst price targets and daily prices.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute `tgt0=log(target_median/close)`, demean by industry, and apply lagged regression coefficients when new targets update.
- Exit conditions: Forecasts are recalculated from the target data; no fixed exit is specified.
- Trade management: Production model stores coefficients and updates daily signals.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily data; signal changes when analyst targets are revised.
- reported_metrics: none reported
- notes: No fixed target/stop is specified by the alpha module.
- sources: [noterminusgit/statarb — Target/close deviation signal and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/target.py#L106-L353); [noterminusgit/statarb — Production price-target implementation](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/prod_tgt.py#L131-L437)

### G152 | Beta-Adjusted Analyst Rating and Price-Move Interaction
- priority: Low — The combined alternative-data/return signal is specific but depends on specialized point-in-time data and fitted coefficients.
- direction: long/short | market: US equities | timeframe: Daily and intraday estimates (intraday-adaptable)
- indicators: Analyst consensus rating level/change, beta-adjusted stock returns, rating stability
- Overview: This strategy combines stable analyst-rating information with beta-adjusted price moves, separating stocks with expanding analyst coverage from stale ratings. It uses multi-lag regression to forecast returns.
- Stock or ETF Criteria: US equities with IBES ratings, predicted beta, market cap, and industry classifications.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Scale beta-adjusted returns by rating signals and retain observations with increasing rating-dispersion/coverage (`std_diff>0`); demean within industry and fit lagged forecasts.
- Exit conditions: Positions are determined from daily/intraday alpha forecasts; no fixed exit is specified.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily data are native; the module also supports intraday inputs.
- reported_metrics: none reported
- notes: Rating stability and beta adjustment are model features; no standalone threshold or portfolio sizing is specified.
- sources: [noterminusgit/statarb — Rating stability, beta-adjusted return interaction, and forecast model](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/analyst_badj.py#L102-L355)

### G153 | Coverage-Filtered Asymmetric Analyst Rating Revisions
- priority: Low — The feature pipeline is explicit but the analyst data are specialized and the signal is not independently validated.
- direction: long/short | market: US equities | timeframe: Daily forecast from analyst-rating events (daily-native)
- indicators: IBES rating changes, rating dispersion changes, signed nonlinear transforms
- Overview: These variants filter rating changes to periods when analyst-rating dispersion/coverage increases, then fit separate upgrade and downgrade regressions. A production variant cubes the rating change to amplify large revisions.
- Stock or ETF Criteria: US equities with IBES rating history and rating-dispersion time series.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Set rating-change signal to zero unless `std_diff>0`; `rating_diff_updn.py` uses sign(x)·x² and separate up/down models, while `prod_rtg.py` uses rating_diff_mean³ with a 720-day fitting window.
- Exit conditions: Daily lagged forecasts are applied with fitted coefficients; no discrete exit or holding limit is specified.
- Trade management: Forecast factors feed the portfolio optimizer; production coefficients are stored and reused.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily factor forecasts updated on analyst event dates.
- parameter_variants: Linear coverage-filtered rating change; sign(x)·x² with asymmetric fits; production x³ signal.
- reported_metrics: none reported
- notes: The repository has several parameter/transform variants; grouped here by the shared rating-revision approach.
- sources: [noterminusgit/statarb — Coverage filter and lagged rating-change forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/rating_diff.py#L97-L303); [noterminusgit/statarb — Signed-square transform and asymmetric upgrade/downgrade regression](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/rating_diff_updn.py#L111-L392); [noterminusgit/statarb — Production cubic rating-change signal and fit/predict modes](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/prod_rtg.py#L114-L359)

### G154 | EWMA Analyst-Rating Change Mean Reversion
- priority: Low — Alternative analyst data offer a behavioral thesis, but data access, point-in-time history and costs are unspecified.
- direction: long/short | market: US equities | timeframe: Daily forecast from analyst-rating events (daily-native)
- indicators: IBES consensus rating changes, EWMA and industry classifications
- Overview: The strategy squares an EWMA of analyst rating changes and demeans it by industry, treating large consensus shifts as a contrarian stock-specific signal. The code fits its relationship to forward returns rather than imposing a fixed RSI-like entry threshold.
- Stock or ETF Criteria: Stocks with IBES analyst-rating history and Barra industry classification.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute the EWMA of `rating_diff_mean`, square the result, and subtract the same-date industry mean; fit the forecast coefficients on the pre-middate sample.
- Exit conditions: Recompute forecasts from rating updates; discrete exits and trade duration are delegated to the portfolio optimizer.
- Trade management: Daily factor forecasts feed the bsim portfolio optimizer; no direct stop or limit rule.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily price data with event-driven analyst updates.
- reported_metrics: none reported
- notes: An industry-demeaned rating-change factor is not an executable direction threshold by itself.
- sources: [noterminusgit/statarb — Rating data, EWMA/squared signal, industry demeaning and regression forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/analyst.py#L62-L231)

### G155 | Hard-to-Borrow Fee Crowding Signal
- priority: Low — Borrow fees have a structural crowding thesis, but specialized data, availability and costs are significant uncertainties.
- direction: long/short | market: US equities | timeframe: Daily borrow-fee observations (daily-native)
- indicators: Stock-loan fee rate, winsorized and lagged
- Overview: High borrow fees are treated as a crowded-short/supply-scarcity signal that can precede a short squeeze or mean reversion. The source regresses lagged fee-rate observations against forward returns.
- Stock or ETF Criteria: Stocks with daily borrow-fee and locate data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Winsorize daily `fee_rate`, form lagged values across the configured horizon, and fit regression coefficients to forward returns.
- Exit conditions: Daily alpha forecast only; explicit entry, exit, and borrow recall handling are not specified in the module.
- Trade management: Forecast is consumed by the portfolio optimizer; borrow availability/recall handling is not detailed.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily factor with borrow-fee data.
- reported_metrics: none reported
- notes: A fee-rate forecast is not itself a direct short-sale instruction.
- sources: [noterminusgit/statarb — Borrow-fee signal construction and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/htb.py#L53-L177)

### G156 | Scheduled-News Straddle Spike (EarnForex News Trader EA)
- priority: Low — Execution-dependent event play outside the daily-swing scope; documented for the calendar-event family.
- direction: long/short | market: FX (MT4/MT5, C# port) | timeframe: seconds around release (intraday-only)
- indicators: Scheduled news time input; optional ATR-based levels; spread fuse
- Overview: Opens a trade (random, chosen, or both directions) exactly SecondsBefore (default 18s) before a scheduled high-impact news release, with fixed SL/TP (15/75 pts default), optional breakeven/trailing, spread fuse, and auto-close X seconds after. The classic 'news spike' play: either catch the initial impulse or straddle both sides - its edge depends entirely on broker execution during the spike.
- Stock or ETF Criteria: High-impact scheduled releases (NFP, CPI, rate decisions) on liquid FX pairs.
- Stocks to avoid and known failure conditions: Spread fuse exists because spreads explode at news - slippage is the entire game; brokers often requote/freeze; two-sided straddle gets stopped both ways in whipsaw releases.
- Entry conditions: X seconds before NewsTime: open chosen direction (or both legs).
- Exit conditions: Fixed SL/TP; optional trailing/breakeven; CloseAfterSeconds time-out.
- Trade management: Single scheduled event per run; both-direction option.
- Target: 75-point TP default (5:1 vs 15-pt SL).
- Risk factors and risk management: SpreadFuse blocks trades if spread >= SL; manual event list required (no calendar feed).
- Best known candlestick timeframe(s): Seconds-level event trading; no daily-bar analog (daily news drift is a different, weaker effect).
- reported_metrics: none reported
- notes: Conceptually adjacent to S081 (pre-FOMC drift) but microsecond-scale.
- sources: [EarnForex/News-Trader — inputs + order logic](https://github.com/EarnForex/News-Trader/blob/005b9949117f74e8ca2726af8b39f34eee36afe3/NewsTrader.mq4); [EarnForex/News-Trader — C# port](https://github.com/EarnForex/News-Trader/blob/005b9949117f74e8ca2726af8b39f34eee36afe3/NewsTrader.cs)

### G157 | Sell-Side Analyst Estimate Revision Forecast
- priority: Low — The data have a plausible underreaction thesis, but source does not provide a cost-aware independent test.
- direction: long/short | market: US equities | timeframe: Daily forecasts at analyst-estimate events (daily-native)
- indicators: SAL mean estimate changes, median estimate level, estimate dispersion
- Overview: The signal normalizes changes in sell-side consensus estimates by the median estimate level, filters for increasing dispersion, and separately fits positive and negative revisions. This targets information diffusion and underreaction to analyst updates.
- Stock or ETF Criteria: US equities with sell-side estimate history and daily prices.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute `SAL_diff_mean/SAL_median` only when `SAL_std` increases (`std_diff>0`); estimate separate lag coefficients for positive/negative revisions. The production variant fits a 720-day lookback.
- Exit conditions: Daily forecasts are applied with stored coefficients; no fixed price exit is specified.
- Trade management: Production version loads saved coefficients and emits daily alpha forecasts.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily bars with event-driven estimate updates.
- reported_metrics: none reported
- notes: The `ebs.py` filename is misleading; the module implements SAL analyst-estimate revisions.
- sources: [noterminusgit/statarb — SAL estimate signal, dispersion filter and asymmetric fit](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/ebs.py#L73-L294); [noterminusgit/statarb — Production estimate-revision signal and coefficient workflow](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/prod_sal.py#L106-L390)

### G158 | Twitter Sentiment Stock-Movement Prediction
- priority: Low — Canonical sentiment implementation; grouped with attention family.
- direction: long/short | market: equities with Twitter coverage | timeframe: 1d (daily-native)
- indicators: Twitter sentiment aggregation -> next-day direction
- Overview: Analyzes stock-related tweets to predict next-day movement - classic social-media sentiment alpha. The attention/sentiment thesis is real but retail-sentiment edges decay fast and spam is heavy.
- Stock or ETF Criteria: Liquid names with tweet volume.
- Stocks to avoid and known failure conditions: Bot/spam contamination; sentiment sign instability.
- Entry conditions: Sentiment score > threshold -> long (and short variants).
- Exit conditions: Next-day resolution.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Overlap S056-S060 sentiment family.
- sources: [jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis — sentiment->prediction (+3 more verified paths)](https://github.com/jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis/blob/cae7dd849ec625c835f36bf6e51f29539f6b58da/Code/Project/StockPrediction.py); [jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis — Code/Project/NaiveBayes.py](https://github.com/jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis/blob/cae7dd849ec625c835f36bf6e51f29539f6b58da/Code/Project/NaiveBayes.py); [jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis — Code/Project/Multiclass_SVM.py](https://github.com/jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis/blob/cae7dd849ec625c835f36bf6e51f29539f6b58da/Code/Project/Multiclass_SVM.py)


## Supervised Machine Learning Prediction

### G159 | Machine-Learning-for-Trading (Stefan Jansen) - Book Strategy Compendium
- priority: High — Highest-quality curated ML-trading compendium available; the LightGBM alpha-factor ranker and FinBERT sentiment strategies are directly testable.
- direction: long/short | market: US equities + FX + futures | timeframe: 1d mostly (daily-native)
- indicators: Full book coverage: factor engineering (alpha factors notebook), tree ensembles (LightGBM/XGBoost alpha models), linear models, NLP sentiment (FinBERT, doc2vec), topic models, word embeddings on filings/news, deep learning seq models, RL (gym envs, DQN/PPO), event-driven backtester
- Overview: The third-edition 'Machine Learning for Trading' code (615 candidate files): a complete tour from feature engineering (alpha factor research with Alphalens), through tree/linear/deep/NLP models predicting returns, to RL agents and event-driven backtesting. The extractable strategies: gradient-boosted alpha-factor rankers on equity factors, NLP sentiment on 10-K/news (FinBERT), cointegration pairs, and an RL gym - each with worked notebooks and evaluation code.
- Stock or ETF Criteria: US equities (various datasets bundled).
- Stocks to avoid and known failure conditions: Book code is per-chapter; datasets age; needs assembly into a coherent strategy.
- Entry conditions: Per model: alpha-factor score ranking (top decile long), sentiment-driven long/short, pairs z-score.
- Exit conditions: Factor-decay-based rebalance; pairs on spread reversion.
- Trade management: Long/short decile portfolios; Alphalens IC evaluation.
- Target: Cross-sectional alpha.
- Risk factors and risk management: Standard ML pitfalls addressed (purged CV, leakage).
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- external_refs: Stefan Jansen, Machine Learning for Algorithmic Trading (3rd ed.)
- notes: Anchor repo for the supervised-ML category; individual chapters map to other records.
- sources: [stefan-jansen/machine-learning-for-trading — book code (+1 more verified paths)](https://github.com/stefan-jansen/machine-learning-for-trading/blob/30db4f00946ce91fbc98fef18582b8cbc3fd8afd/README.md); [stefan-jansen/machine-learning-for-trading — data](https://github.com/stefan-jansen/machine-learning-for-trading/blob/30db4f00946ce91fbc98fef18582b8cbc3fd8afd/data)

### G160 | QLib Model Zoo on Daily Alpha Factors (LightGBM/MLP/LSTM/ALSTM/TFT/Transformer)
- priority: High — Published model-vs-feature benchmarks on daily bars - the fastest route to a credible ML ranker.
- direction: long/short | market: CN A-shares (CSI300/500) | timeframe: 1d (daily-native)
- indicators: Alpha158/360 features -> gradient boosting, MLP, LSTM, ALSTM, GRU, TFT, Transformer, GATs, Master, Localformer, TRA, KRNN, Sandwich, SFM, TCN, DNN, CatBoost, XGBoost, DoubleEnsemble
- Overview: qlib ships ~25 ready models trained on the Alpha factor sets with published IC/ICIR benchmark tables. The reusable insight: ranking metrics (IC/ICIR/RankIC) per model are published for CSI300/500, so you can pick the best model+feature combo by evidence instead of guessing - LightGBM and DoubleEnsemble are strong cheap baselines; attention/LSTM variants add modest IC.
- Stock or ETF Criteria: Alpha158-featured daily panels.
- Stocks to avoid and known failure conditions: Deep models show marginal IC gains over LightGBM at much higher cost; results are A-share specific.
- Entry conditions: Daily top-k on model-predicted scores.
- Exit conditions: Rebalance.
- Trade management: qlib workflow rolling retrain.
- Target: IC/RankIC-driven alpha.
- Risk factors and risk management: Rolling retraining mitigates drift.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: Benchmark IC/ICIR tables in repo (repo-claimed, unverified; e.g., LightGBM RankIC ~0.04-0.05 on CSI300).
- notes: Use RankIC comparison to select model; execution-RL submodule is execution, out of scope.
- sources: [microsoft/qlib — model impls + benchmark tables (+5 more verified paths)](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/examples/benchmarks/README.md); [microsoft/qlib — qlib/contrib/model/gbdt.py](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/qlib/contrib/model/gbdt.py); [microsoft/qlib — qlib/contrib/model/pytorch_lstm.py](https://github.com/microsoft/qlib/blob/54355232463878d2eebb91fe0ee5fa7fa1f5976c/qlib/contrib/model/pytorch_lstm.py)

### G161 | CUSUM-Filtered Event + Triple-Barrier-Label ML Long/Short Pipeline
- priority: Medium — Methodology-grade event-ML pipeline; genuine edge needs real revenue/alternative data - a strong candidate once data exists.
- direction: long/short | market: US equities EOD (simulated + SPY) | timeframe: 1d (daily-native)
- indicators: Symmetric CUSUM filter on YoY revenue changes (event triggers); triple-barrier labels (vol-scaled upper/lower + time); engineered features; RF/ML classifier
- Overview: Implements the Lopez de Prado-style event-driven ML pipeline end to end: trigger events only when YoY revenue changes breach a symmetric CUSUM threshold (events, not bars, drive the problem), label each event by the triple-barrier method (first touch of +z*vol upper, -z*vol lower, or time barrier -> +1/-1/0), build features, train an ML classifier with repeated K-fold CV, then simulate a portfolio on model signals. The value is the event+labeling machinery that turns sparse fundamental surprises into a tradeable supervised problem.
- Stock or ETF Criteria: Stocks with revenue history (alternative-data matrix) + EOD prices; simulated dataset included.
- Stocks to avoid and known failure conditions: Labels are volatility-relative so low-vol names generate ambiguous 0 labels; event scarcity (revenue is quarterly) means few trades per name - needs a wide universe; leakage risk if features use post-event data (average-uniqueness helper included).
- Entry conditions: Long when model predicts +1 for a new CUSUM event (probability above threshold); short/-1 symmetric.
- Exit conditions: Position resolved at the triple-barrier horizon used for training (upper/lower/time); signal-driven.
- Trade management: Event-uniformity weighting via average uniqueness; CV-validated model; portfolio sim allocates across open signals.
- Target: First-barrier touch or time barrier (z-scaled vol window).
- Risk factors and risk management: Model overfit mitigated by repeated K-fold CV; event overlap handled by uniqueness weights.
- Best known candlestick timeframe(s): Daily bars (events are quarterly-ish but positions resolve over days-weeks).
- reported_metrics: none reported
- external_refs: Lopez de Prado, Advances in Financial Machine Learning (CUSUM filter, triple barrier, uniqueness)
- notes: Includes average-uniqueness weighting (sample weights) - rare in educational repos.
- sources: [chrisconlan/algorithmic-trading-with-python — CUSUM events](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/listings/chapter_7/7_1_symmetric_cusum_filter_on_revenue.py); [chrisconlan/algorithmic-trading-with-python — triple-barrier labeling](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/listings/chapter_7/7_2_computing_triple_barrier_labels.py); [chrisconlan/algorithmic-trading-with-python — model pipeline](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/listings/chapter_7/7_6_machine_learning_pipeline.py); [chrisconlan/algorithmic-trading-with-python — portfolio sim](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/listings/chapter_7/7_7_simulation_with_machine_learning_model.py)

### G162 | Dahshan228 FTMO Bot - LightGBM Equity Signal + BTC/FX Trend (2015-2025 validated)
- priority: Medium — Documented risk-compliance + LightGBM signal - worth extracting the model and risk layer.
- direction: long/short | market: equity + BTC + forex (FTMO prop) | timeframe: daily-ish (daily-native)
- indicators: LightGBM classification on engineered features + trend-following modules, FTMO-compliant risk
- Overview: Multi-strategy prop-firm bot: a LightGBM classifier generates equity long/short signals while separate trend modules handle BTC/forex, all under FTMO drawdown rules. Claims 1600+ backtests 2015-2025 and demo live-validation - unusually documented risk-compliance discipline for a repo bot.
- Stock or ETF Criteria: FTMO-allowed instruments.
- Stocks to avoid and known failure conditions: Prop-challenge tuning surface; claims unverified.
- Entry conditions: LightGBM signal + trend modules.
- Exit conditions: Model/trend exits; FTMO DD caps.
- Trade management: Prop-rule sizing/daily-loss.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: FTMO compliance layer (daily loss, max DD).
- Best known candlestick timeframe(s): Daily-ish.
- reported_metrics: Claims backtested 2015-2025, 1600+ tests, live-validated on demo (repo-claimed, unverified).
- notes: Merge with supervised group; risk layer notable.
- sources: [Dahshan228/FTMO-TradingBot — backtest/walkforward/risk scripts (+5 more verified paths)](https://github.com/Dahshan228/FTMO-TradingBot/blob/8d1ecdff8d1b933870273794bc5b63216b4a3167/scripts/combined_backtest.py); [Dahshan228/FTMO-TradingBot — scripts/mean_rev_backtest.py](https://github.com/Dahshan228/FTMO-TradingBot/blob/8d1ecdff8d1b933870273794bc5b63216b4a3167/scripts/mean_rev_backtest.py); [Dahshan228/FTMO-TradingBot — scripts/fib_backtest.py](https://github.com/Dahshan228/FTMO-TradingBot/blob/8d1ecdff8d1b933870273794bc5b63216b4a3167/scripts/fib_backtest.py)

### G163 | Pattern-Similarity k-NN Forecast (normalized %change shape matching)
- priority: Medium — Genuinely different forecasting approach (analog/k-NN on shape) worth a daily-bar test; the similarity library is easy to replicate.
- direction: long/short | market: FX (GBP/USD tick data) | timeframe: tick (intraday-adaptable)
- indicators: 30-point percent-change pattern from an averaged price line; similarity matching vs historical patterns; mean of matched outcomes as forecast
- Overview: sentdex's classic pattern-recognition approach: compress the last 30 price changes into a normalized %change pattern, find the most similar historical patterns, and predict the average of their short-term outcomes (with an optional accuracy-tracked threshold). Effectively a hand-rolled k-nearest-neighbor forecast on price-shape. The idea: similar local tape tends to resolve similarly.
- Stock or ETF Criteria: GBP/USD tick series; any liquid instrument with enough history to build a pattern library.
- Stocks to avoid and known failure conditions: Pattern library is stationary - regime shifts make old analogs meaningless; %change normalization ignores vol scale; similarity metrics are crude (euclidean on raw %changes).
- Entry conditions: Trade in the direction of the mean outcome of the N most similar historical patterns, when predicted magnitude exceeds a threshold (pattern recognition + outcome-average logic across machFX* scripts).
- Exit conditions: Fixed horizon implied by the stored outcome window; accuracy-threshold gating in later scripts.
- Trade management: Accuracy tracking per pattern used as an execution filter in later versions (machFX11-16 add performance/accuracy gating).
- Target: Mean forward outcome of matched patterns.
- Risk factors and risk management: No explicit risk layer in the scripts; rely on prediction-accuracy gate.
- Best known candlestick timeframe(s): Tick native. Daily adaptation: match the last ~10-20 daily %changes against history and trade the mean next-day outcome - testable but unproven at daily scale.
- reported_metrics: none reported
- external_refs: pythonprogramming.net pattern-recognition trading series
- sources: [PythonProgramming/Pattern-Recognition-for-Forex-Trading — patternStorage 30-point percent-change patterns + similarity matching](https://github.com/PythonProgramming/Pattern-Recognition-for-Forex-Trading/blob/19b9d2499c57ed5be9552952f6c0af0ba90e5650/machFX16.py); [PythonProgramming/Pattern-Recognition-for-Forex-Trading — project link](https://github.com/PythonProgramming/Pattern-Recognition-for-Forex-Trading/blob/19b9d2499c57ed5be9552952f6c0af0ba90e5650/README.md)

### G164 | Surpriver - Anomaly-Detection Pre-Move Screener
- priority: Medium — Distinctive anomaly-anticipation mechanism usable as a universe filter for a breakout strategy.
- direction: long | market: US equities + crypto | timeframe: 1d (daily-native)
- indicators: Unsupervised anomaly detection (IsolationForest-ish) on return/volatility/volume features; similarity clustering of 'about to move' stocks
- Overview: Finds 'big movers before they move': computes feature vectors of recent return/volatility/volume patterns, detects anomalous (pre-move) signatures with unsupervised models, and clusters stocks whose current state resembles past pre-move states. Distinct mechanism - anomaly-conditioned anticipation rather than factor ranking.
- Stock or ETF Criteria: Liquid US equities; crypto variant included.
- Stocks to avoid and known failure conditions: Anomaly != direction - it flags volatility about to expand, not which way; needs a direction overlay.
- Entry conditions: Screen for top anomaly-score names; enter on direction confirmation (author suggests breakout of the detected setup).
- Exit conditions: Per overlay.
- Trade management: Screener, not a portfolio.
- Target: Volatility expansion capture.
- Risk factors and risk management: False positives dominate; no explicit sizing.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- notes: Use as a screening layer, not standalone.
- sources: [tradytics/surpriver — anomaly detection + feature generation (+3 more verified paths)](https://github.com/tradytics/surpriver/blob/52417660e44c9533ba4dfc49f0dcb45c885d05dc/detection_engine.py); [tradytics/surpriver — feature_generator.py](https://github.com/tradytics/surpriver/blob/52417660e44c9533ba4dfc49f0dcb45c885d05dc/feature_generator.py); [tradytics/surpriver — data_loader.py](https://github.com/tradytics/surpriver/blob/52417660e44c9533ba4dfc49f0dcb45c885d05dc/data_loader.py)

### G165 | ai_quant_trade - One-Stop AI Quant Platform Strategy + Factor-Mining + LLM Set
- priority: Medium — Factor-mining + LLM-trading modules worth a look; otherwise curriculum.
- direction: long/short | market: multi (CN-heavy) | timeframe: 1d (daily-native)
- indicators: Example strategies + factor mining + ML/DL/RL + LLM trading modules
- Overview: A very large one-stop platform (Jupyter) covering example strategies, factor mining, ML/DL/RL agents, LLM-based trading, and deployment. The extractable value is the factor-mining and LLM-trading modules plus its worked example strategies - grouped here rather than per-file since the repo is a curriculum.
- Stock or ETF Criteria: CN market-heavy examples.
- Stocks to avoid and known failure conditions: Curriculum breadth over depth; many modules are demos.
- Entry conditions: Per module strategy/factor.
- Exit conditions: Per module.
- Trade management: Platform-level.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily primarily.
- reported_metrics: none reported
- notes: Grouped platform entry.
- sources: [charliedream1/ai_quant_trade — platform modules](https://github.com/charliedream1/ai_quant_trade/blob/035f5423899462f68989efef4c11c04dceb870bd/README.md)

### G166 | robertmartin8 MachineLearningStocks - Fundamental+Technical Sklearn Prediction
- priority: Medium — Clean canonical sklearn stock-picker - good ML baseline.
- direction: long | market: S&P 500 | timeframe: 1d features / monthly-ish evaluation (daily-native)
- indicators: Fundamental ratios + technical features -> sklearn classifiers (random forest default) predicting next-period outperformers
- Overview: Classic sklearn stock-picker: builds a feature matrix of fundamentals + technicals for S&P 500 names, trains a classifier on labeled outperform/underperform, and holds the predicted top names. Straightforward and reproducible - the canonical 'ML stock picker' pattern.
- Stock or ETF Criteria: S&P 500 with fundamentals.
- Stocks to avoid and known failure conditions: Class imbalance and survivorship in S&P constituents; monthly-ish signal decay.
- Entry conditions: Top predicted probability names, equal weight.
- Exit conditions: Rebalance on next prediction cycle.
- Trade management: Equal-weight top-N.
- Target: Cross-sectional outperformance.
- Risk factors and risk management: Model + data leakage care needed.
- Best known candlestick timeframe(s): Daily features, periodic rebalance.
- reported_metrics: none reported
- notes: Baseline for the supervised-ML group.
- sources: [robertmartin8/MachineLearningStocks — model + backtest + feature parsing (+3 more verified paths)](https://github.com/robertmartin8/MachineLearningStocks/blob/1c3712af741a9a030edb678b37ef18d1d2db9147/stock_prediction.py); [robertmartin8/MachineLearningStocks — backtesting.py](https://github.com/robertmartin8/MachineLearningStocks/blob/1c3712af741a9a030edb678b37ef18d1d2db9147/backtesting.py); [robertmartin8/MachineLearningStocks — parsing_keystats.py](https://github.com/robertmartin8/MachineLearningStocks/blob/1c3712af741a9a030edb678b37ef18d1d2db9147/parsing_keystats.py)

### G167 | Classical ML Stock-Predictor Set (SVM/KNN/GP/AdaBoost/QDA/ARIMA/LSTM course repos)
- priority: Low — Approach class with weak OOS edge; keep robertmartin8 as the clean baseline.
- direction: long/short | market: equities | timeframe: 1d (daily-native)
- indicators: Statistical indicators -> supervised classifier/regressor per repo
- Overview: Grouped entry for the many 'predict stock with sklearn/ARIMA/NN' course repos: kennedyCzar (KNN/SVM/GP/AdaBoost/tree/QDA), SravB, kknet (indicator predictors), hklchung, TimRivoli (TF), yacoubb, grananqvist ARIMA, jcwill415, xljiang, lnmangione, Amey-Thakur, hahnicity, shivamakhauri04, LiamConnell, jsgaston LSTM+MT5, Anshupriya2694, sushantsbelapurkar, LastAncientOne, huseinzol05 zoo, Packt x2. Common pattern: technical indicators -> next-day direction classifier. Baseline results are ~coin-flip OOS; useful as the negative-control class and for the feature sets.
- Stock or ETF Criteria: Equities/FX per repo.
- Stocks to avoid and known failure conditions: Direction prediction near 50% OOS; train/test leakage common.
- Entry conditions: Classifier sign -> position.
- Exit conditions: Daily.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Deduplicated supervised family.
- sources: [kennedyCzar/STOCK-RETURN-PREDICTION-USING-KNN-SVM-GUASSIAN-PROCESS-ADABOOST-TREE-REGRESSION-AND-QDA — multi-model](https://github.com/kennedyCzar/STOCK-RETURN-PREDICTION-USING-KNN-SVM-GUASSIAN-PROCESS-ADABOOST-TREE-REGRESSION-AND-QDA/blob/3b0a822b296da7eaf9deb712c906310627360c19/README.md); [huseinzol05/Stock-Prediction-Models — model zoo (+1 more verified paths)](https://github.com/huseinzol05/Stock-Prediction-Models/blob/33266732b0b16188b565e0aeb6b24efa71161f6a/README.md); [huseinzol05/Stock-Prediction-Models — agent](https://github.com/huseinzol05/Stock-Prediction-Models/blob/33266732b0b16188b565e0aeb6b24efa71161f6a/agent)

### G168 | Decision-Tree Leg-Selection for Pair Rotation
- priority: Low — Demonstrates rolling ML pair-selection; the pattern transfers better than the toy feature set.
- direction: long | market: equity pairs (PEP/KO example) | timeframe: 1d (daily-native)
- indicators: Features: leg identity, ATR, leg returns (AKO/APEP); sklearn DecisionTreeRegressor, rolling walk-forward (38 train / 3 test)
- Overview: Extends the buy-the-loser pair rotation with a rolling decision-tree regressor that predicts next-period return for each leg from simple features (which leg, ATR, recent leg returns), then holds the predicted winner. Demonstrates the walk-forward ML pair-selection pattern - the model is retrained every 3 sessions on the trailing 38.
- Stock or ETF Criteria: Pair legs with feature history (ATR, returns).
- Stocks to avoid and known failure conditions: Minimal feature set limits signal-to-noise (author says so); shallow trees on 38-row windows overfit; per-leg market exposure remains.
- Entry conditions: Each step: refit trees on trailing 38 obs, predict each leg's next return, hold the higher-predicted leg.
- Exit conditions: Re-predicted each test step (3-day blocks).
- Trade management: One leg at a time; rolling refit cadence 38/3.
- Target: Predicted-winner leg outperformance.
- Risk factors and risk management: Micro training windows; feature sparsity.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- notes: AssetAllocation.ipynb is optimization-only - logged not recorded.
- sources: [LongOnly/Quantitative-Notebooks — rolling DTR on pair features](https://github.com/LongOnly/Quantitative-Notebooks/blob/9041f80c21b4029afdbd044c60f4b8d85473b70d/DecisionTreeRegressors.ipynb)

### G169 | Dollar-Bar + Triple-Barrier ML Strategy (QuantConnect, MOEX tick)
- priority: Low — Tick-native AFML implementation; methodology overlaps the Conlan record - use the daily version.
- direction: long/short | market: MOEX futures tick data -> dollar bars | timeframe: dollar bars (intraday-only)
- indicators: Dollar-bar resampling; triple-barrier labels; technical indicators on dollar bars; sklearn classifier via GridSearchCV
- Overview: Faithful Lopez-de-Prado microstructure pipeline: raw MOEX ticks -> dollar bars -> triple-barrier labels + indicator features -> grid-searched sklearn classifiers per instrument -> QC/LEAN live strategy trading model predictions with stop-time management. Demonstrates the full data path from ticks to labeled events.
- Stock or ETF Criteria: MOEX futures with tick history; needs the preprocessing pipeline (5 scripts).
- Stocks to avoid and known failure conditions: Tick-data infrastructure heavy; dollar-bar thresholds are instrument-specific; GridSearchCV on financial labels overfits without purged CV (not evident in code).
- Entry conditions: Model prediction (per-instrument classifier) on latest dollar-bar features; long/short per prediction sign.
- Exit conditions: Triple-barrier-consistent stop_time management in OnData.
- Trade management: Per-instrument models; QC portfolio framework.
- Target: Barrier-defined horizons.
- Risk factors and risk management: Grid-search selection bias; data pipeline fragility.
- Best known candlestick timeframe(s): Dollar bars (event-driven); no direct daily analog - the Conlan ch7 pipeline is the daily version.
- reported_metrics: none reported
- external_refs: Lopez de Prado AFML (dollar bars, triple barrier)
- notes: MOEX-specific; WQU capstone project.
- sources: [WQU-MScFE-Capstone-MGS/qc-tick-data-strategies — bar + label pipeline](https://github.com/WQU-MScFE-Capstone-MGS/qc-tick-data-strategies/blob/5574fb47b04fd6cc102f69c421f0ac869a0e2823/4_dollar_bars_triple_barrier_indicators.py); [WQU-MScFE-Capstone-MGS/qc-tick-data-strategies — LEAN strategy with TrainML/GridSearchCV](https://github.com/WQU-MScFE-Capstone-MGS/qc-tick-data-strategies/blob/5574fb47b04fd6cc102f69c421f0ac869a0e2823/QCTickDataStrategy.py)

### G170 | Hurst-Exponent Stationarity Screen + ARIMA Forecast Filter (Indian equities)
- priority: Low — Screening methodology is the only durable idea; ARIMA adds little; minute-native data.
- direction: long | market: Indian equities (111 stocks, minute VWAP) | timeframe: 1min->daily aggregation (intraday-adaptable)
- indicators: Hurst exponent for stationarity classification; ARIMA forecast accuracy
- Overview: Filters a 111-stock Indian universe by the Hurst exponent to retain only mean-reverting (stationary, H<0.5) series, evaluates them over 1-day/2-week/5-day windows to pick the top 25, then uses ARIMA forecasts for direction. The transferable idea is Hurst-based tradability screening BEFORE applying a mean-reversion strategy - only fade what actually reverts.
- Stock or ETF Criteria: Stocks passing H<0.5 on minute VWAP series; top-25 by analysis-window metrics.
- Stocks to avoid and known failure conditions: Hurst is unstable in short windows; 2017-only minute data; ARIMA on raw prices is weak.
- Entry conditions: not specified in source. ARIMA-forecast direction on screened stocks; exact trigger not specified. Suggested: trade only H<0.5 names, use ARIMA sign.
- Exit conditions: not specified in source.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Minute data aggregated to daily/2-week/5-day windows.
- reported_metrics: none reported
- notes: Hurst-screen concept also appears in stephenkyang pairs pipeline - consistent motif.
- sources: [imad08/Mean-Reversion-Trading-Strategy — Hurst screening](https://github.com/imad08/Mean-Reversion-Trading-Strategy/blob/6ad33b2b44d5310a514a3afbcb2824caf29f726d/Hurst_value_.ipynb); [imad08/Mean-Reversion-Trading-Strategy — ARIMA prediction](https://github.com/imad08/Mean-Reversion-Trading-Strategy/blob/6ad33b2b44d5310a514a3afbcb2824caf29f726d/arima.ipynb); [imad08/Mean-Reversion-Trading-Strategy — pipeline](https://github.com/imad08/Mean-Reversion-Trading-Strategy/blob/6ad33b2b44d5310a514a3afbcb2824caf29f726d/README.md)

### G171 | PCA-Reduced SVR Pair Fair-Value Voting
- priority: Low — The model is an explicit daily pair-fair-value method, but evidence is limited and parameter selection is small-sample.
- direction: long/short | market: US equities; pair universe selected in notebook | timeframe: 1d (daily-native)
- indicators: PCA components, pairwise price history, support-vector-regression predictions
- Overview: The notebook trains multiple SVR models to estimate a pair leg's fair value from the other leg and uses a weighted vote across model predictions. The vote fades the asset when observed price is above predicted fair value and favors it when below.
- Stock or ETF Criteria: Pairs selected from the notebook's stock universe; no stable universe rule specified.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For each pair and date, compute votes as −sign(observed price − each SVR prediction), then take the dot product with model-vote weights. The notebook tests a 20-session training lookback and PCA component counts 1–3.
- Exit conditions: The source evaluates directional votes and applies a 0.2% fee on position reversals; a complete fixed exit threshold is not specified in source.
- Trade management: Long/short vote is converted into a directional position; 0.2% fee is charged when the position reverses.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Model parameters are selected on a short sample; the code's parameter search and backtest reuse are not a robust walk-forward design.
- Best known candlestick timeframe(s): Daily close series are used; the approach is technically daily-adaptable but forecasts a pair fair value.
- reported_metrics: none reported
- notes: The notebook does not provide a market-neutral hedge-ratio sizing rule for the vote portfolio.
- sources: [financialnoob/pairs_trading — PCA/SVR fair-value voting, 20-day lookback and transaction-cost calculation; code cells 10–16 and 22](https://github.com/financialnoob/pairs_trading/blob/88e94ea51fbab6397efb021e03334c6f596515fb/7.pair_trading.svr_v1.ipynb)

### G172 | SVM 90-Day Multi-Feature Direction Classifier
- priority: Low — The feature/label pipeline is explicit, but the sample is limited and the repo does not establish cost-aware out-of-sample performance.
- direction: long/short | market: Multi-asset dataset; exact asset list not specified in source | timeframe: Daily inputs; 21-session holding/rebalance cadence (daily-native)
- indicators: 90-day return mean, volatility, skewness, kurtosis, three 30-day means, and asset-class one-hot features
- Overview: An RBF-kernel SVM classifies each asset's next 21-session cumulative return as positive or negative using rolling 90-day distributional features. The predicted sign is combined with volatility/risk-parity allocation.
- Stock or ETF Criteria: Multi-asset daily return histories with asset-class labels.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Train on the first 40% of the historical sample using 90-day windows and 21-day forward compounded returns; label nonnegative future return +1 and negative return 0. At each 21-session date, predict the next-direction class from the prior 90 sessions.
- Exit conditions: Hold the predicted long/short signal until the next 21-session rebalance; no per-position stop or price target is specified.
- Trade management: Main.m tests equal, volatility-parity and risk-parity weights; the model targets 10% annualized volatility and charges 0.1% portfolio cost.
- Target: Rebalance every 21 sessions; no fixed profit target.
- Risk factors and risk management: The model uses a fixed first-40%-of-history training split and a small sample; no independent modern OOS results are provided.
- Best known candlestick timeframe(s): Daily signals and approximately monthly holds are native; not a short-swing multi-trade-per-week system.
- reported_metrics: none reported
- notes: SVM classifications are direction labels; risk-parity and volatility-targeting are allocation variants, not separate prediction approaches.
- sources: [blacksouane/QARMII_project — Feature construction, training split, forward 21-session label, and RBF SVM](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/SVM/SVM_Model.m#L42-L111); [blacksouane/QARMII_project — Monthly signal generation, portfolio weights, and volatility targeting](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Models/SVM/SVM_Strategy.m#L1-L56); [blacksouane/QARMII_project — Prediction-to-signal mapping](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Functions%20and%20Scripts/Signal/SVM_Signal.m); [blacksouane/QARMII_project — SVM strategy configurations and 0.1% portfolio cost](https://github.com/blacksouane/QARMII_project/blob/dc023aaba9e37388c196e081f63a556ec00b9fd8/Main.m#L586-L675)

### G173 | intelligent-trading-bot - Feature-Engineered Signal Generation (asavinov)
- priority: Low — Feature-engineering pattern; crypto intraday.
- direction: long/short | market: crypto | timeframe: intraday->higher (intraday-adaptable)
- indicators: Rolling feature engineering -> score-based signal generation with automated feature selection
- Overview: Trading bot built around systematic feature engineering: generates many rolling technical features, scores them, and produces trade signals - a pipeline emphasizing feature generation/selection rather than a fixed indicator set.
- Stock or ETF Criteria: Crypto pairs.
- Stocks to avoid and known failure conditions: Feature proliferation = overfit surface.
- Entry conditions: Score-threshold signals on engineered features.
- Exit conditions: Signal decay/threshold.
- Trade management: Bot-managed.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday; feature-gen pattern transfers to daily.
- reported_metrics: none reported
- notes: Merge with supervised-ML group.
- sources: [asavinov/intelligent-trading-bot — features + signals (+1 more verified paths)](https://github.com/asavinov/intelligent-trading-bot/blob/0f8748d21906d37b7d33de9278f4742b8f419456/common/); [asavinov/intelligent-trading-bot — service/App.py](https://github.com/asavinov/intelligent-trading-bot/blob/0f8748d21906d37b7d33de9278f4742b8f419456/service/App.py)


## Deep Learning Sequence Models

### G174 | Encoder-Decoder LSTM Multi-Step Pair-Spread Forecast
- priority: Low — The multi-step architecture is a distinct forecasting approach, but generalization and net cost robustness are not established.
- direction: long/short | market: Equities; pair universe chosen by cointegration/clustering | timeframe: Daily (daily-native)
- indicators: Normalized pair-spread sequences, encoder-decoder LSTM predictions at t and t+1
- Overview: An encoder-decoder LSTM predicts the current and next pair spread, then forecasted changes are thresholded using training-period quantiles. This is a distinct multi-step architecture from the repository's single-output recurrent model.
- Stock or ETF Criteria: Cointegrated equity pairs with daily spread histories and training/validation/test partitions.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Train on normalized spread sequences; use 10th/90th quantiles of training spread changes (lag=1, multistep=1) to set directional positions and shift trades to the next session.
- Exit conditions: Close or reverse when forecasted spread-change direction no longer passes its quantile threshold; no independent stop is specified.
- Trade management: Beta-weighted pair returns; the Trader class applies transaction costs.
- Target: No fixed price target; exposure follows predicted spread change.
- Risk factors and risk management: Model selection and validation-period profitable-pair filtering can bias test results if reused.
- Best known candlestick timeframe(s): Daily bars are used.
- reported_metrics: none reported
- notes: The notebook includes an active-pair selection variant based on validation-period profitability.
- sources: [simaomsarmento/PairsTrading — Forecast quantiles and position construction](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/classes/class_ForecastingTrader.py#L52-L169); [simaomsarmento/PairsTrading — Encoder-decoder model and multi-step prediction](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/classes/class_ForecastingTrader.py#L791-L851); [simaomsarmento/PairsTrading — Encoder-decoder LSTM training, validation, and test application](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/notebooks/PairsTrading-Forecasting_2009_2019.ipynb); [simaomsarmento/PairsTrading — Encoder-decoder model training](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/training/encoder_decoder_trainer.py)

### G175 | LSTM/NN Return-Prediction Alpha with Portfolio Sim (MS&E 448)
- priority: Low — Standard seq-model alpha scaffold; grouped under DL approach with other course repos.
- direction: long | market: US equities | timeframe: 1d (daily-native)
- indicators: LSTM + NN + linear-regression alpha models over daily features; portfolio sim + stoploss
- Overview: Course project pipeline: preprocessing -> LSTM and NN alpha models predicting next returns -> portfolio simulation with a stop-loss module and volatility analysis. Multiple model variants (LSTM_model, nn_alpha, regression_model, joe_main_nn single/multi-model) compared against a baseline strategy.
- Stock or ETF Criteria: US equities daily.
- Stocks to avoid and known failure conditions: Course scale: single-model predictions without cost-aware evaluation in the sim; trained weights committed (model_dict.pt) - no retraining protocol documented.
- Entry conditions: Model-predicted return sign/magnitude -> position.
- Exit conditions: Stop-loss module + signal flip.
- Trade management: portfolio_sim.py sizing; stoploss.py.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Stop-loss module; volatility plot diagnostics.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- notes: Merge with the 'LSTM/NN next-day direction' grouped approach at assembly.
- sources: [jctags/mse448-trend-following — LSTM alpha](https://github.com/jctags/mse448-trend-following/blob/583496ce7931ba3502b0dbe60127e6146ef0c8f4/LSTM_model.py); [jctags/mse448-trend-following — NN alpha](https://github.com/jctags/mse448-trend-following/blob/583496ce7931ba3502b0dbe60127e6146ef0c8f4/nn_alpha.py); [jctags/mse448-trend-following — sim](https://github.com/jctags/mse448-trend-following/blob/583496ce7931ba3502b0dbe60127e6146ef0c8f4/portfolio_sim.py)

### G176 | RNN Forecasted Pair-Spread Momentum
- priority: Low — The sequence-model approach has explicit train/test windows but no evidence of robust net performance.
- direction: long/short | market: Equities; pair universe chosen by cointegration/clustering | timeframe: Daily (daily-native)
- indicators: Normalized pair-spread sequence, recurrent-network forecast, training-period spread-change quantiles
- Overview: A recurrent network forecasts the pair spread, and predicted percentage changes are compared with training-period quantiles to determine directional exposure. Pair construction and the forecast-to-position rule are distinct from a conventional spread z-score.
- Stock or ETF Criteria: Cointegrated equity pairs with daily price histories and train/validation/test partitions.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Train a recurrent model on normalized spread sequences; use the default 10th/90th quantiles of positive/negative training spread changes as short/long thresholds. Enter when predicted spread change crosses the relevant threshold and shift the position one session to avoid same-bar execution.
- Exit conditions: Carry a position only while the next predicted spread change retains its direction; the source does not define an independent price stop.
- Trade management: Beta-weighted two-leg returns; the Trader class applies transaction costs; default lag is one session.
- Target: No fixed target; positions follow predicted spread changes.
- Risk factors and risk management: Pair/model selection can overfit and the RNN is evaluated on a limited historical sample.
- Best known candlestick timeframe(s): Daily bars are used.
- reported_metrics: none reported
- notes: The repository also tests ARMA/MLP baselines; this record is limited to its recurrent-network approach.
- sources: [simaomsarmento/PairsTrading — Forecast-change quantile thresholds and position shift](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/classes/class_ForecastingTrader.py#L52-L169); [simaomsarmento/PairsTrading — RNN model implementation](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/classes/class_ForecastingTrader.py#L700-L788); [simaomsarmento/PairsTrading — RNN training, validation, and test application](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/notebooks/PairsTrading-Forecasting_2009_2019.ipynb); [simaomsarmento/PairsTrading — RNN model training implementation](https://github.com/simaomsarmento/PairsTrading/blob/0781877c75673ceca3c61704eee9c9dca9d37b6b/training/rnn_trainer.py)

### G177 | StockFormer - Hybrid Predictive-Coding Trading Machine
- priority: Low — Research architecture; evidence is paper-internal.
- direction: long/short | market: equities/futures | timeframe: 1d (daily-native)
- indicators: Predictive-coding representation + RL trading head
- Overview: PyTorch implementation of 'StockFormer: Learning Hybrid Trading Machines with Predictive Coding' - a representation-learning front (predictive coding) feeding a trading policy. Research-grade seq-model approach with published comparisons.
- Stock or ETF Criteria: Per paper datasets.
- Stocks to avoid and known failure conditions: Research code; training cost and complexity high.
- Entry conditions: Policy output per bar.
- Exit conditions: Policy-driven.
- Trade management: RL-style position actions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- external_refs: StockFormer paper (ICASSP/related venue)
- notes: Merge with DL approach group.
- sources: [gsyyysg/StockFormer — predictive-coding SAC (+3 more verified paths)](https://github.com/gsyyysg/StockFormer/blob/9e8f1ac5f9dbd861f4eda4adefc6add46e3247b8/code/MySAC/SAC/MAE_SAC.py); [gsyyysg/StockFormer — code/MySAC/SAC/policy_transformer.py](https://github.com/gsyyysg/StockFormer/blob/9e8f1ac5f9dbd861f4eda4adefc6add46e3247b8/code/MySAC/SAC/policy_transformer.py); [gsyyysg/StockFormer — code/envs/env_stocktrading_hybrid_control.py](https://github.com/gsyyysg/StockFormer/blob/9e8f1ac5f9dbd861f4eda4adefc6add46e3247b8/code/envs/env_stocktrading_hybrid_control.py)


## Reinforcement Learning Agents

### G178 | FinRL Ensemble DRL Agent (PPO/A2C/DDPG/TD3/SAC) on Dow30 Daily
- priority: Medium — Canonical daily-bar RL pipeline with an ensemble-selection mechanism; the pattern is reusable even if raw edge is thin.
- direction: long | market: US equities (Dow 30) daily + crypto | timeframe: 1d (daily-native)
- indicators: OHLCV + technical indicators + turbulence index + VIX as state; portfolio weights as action
- Overview: The reference financial RL library: trains PPO/A2C/DDPG/TD3/SAC agents on daily bars to output portfolio weights, with a 'turbulence index' regime signal that switches the ensemble model each window. The ensemble idea (validate each agent every window, trade the best) is the useful design; raw DRL performance is typically ~index-level once costs are honest.
- Stock or ETF Criteria: Dow30 daily (demo); any liquid multi-asset universe.
- Stocks to avoid and known failure conditions: DRL overfits to the training window; turbulence filter helps but is not magic; long-only baseline.
- Entry conditions: Agent outputs portfolio weights each day; ensemble picks best-validated agent per window.
- Exit conditions: Daily weight rebalance per agent action.
- Trade management: Turbulence-triggered liquidation option in some envs.
- Target: Portfolio value maximization vs DJIA.
- Risk factors and risk management: Turbulence index as kill-switch.
- Best known candlestick timeframe(s): Daily bars (native env).
- reported_metrics: Paper reports ensemble >= best single agent and ~DJIA-level or modest excess (repo-claimed, unverified).
- external_refs: FinRL library papers (NeurIPS workshops)
- sources: [AI4Finance-Foundation/FinRL — agents + env + ensemble example (+5 more verified paths)](https://github.com/AI4Finance-Foundation/FinRL/blob/7e98df461cbc7772f16a545a2f13b521e5d0b8f4/finrl/agents/elegantrl/models.py); [AI4Finance-Foundation/FinRL — finrl/train.py](https://github.com/AI4Finance-Foundation/FinRL/blob/7e98df461cbc7772f16a545a2f13b521e5d0b8f4/finrl/train.py); [AI4Finance-Foundation/FinRL — examples/FinRL_Ensemble_StockTrading_ICAIF_2020.ipynb](https://github.com/AI4Finance-Foundation/FinRL/blob/7e98df461cbc7772f16a545a2f13b521e5d0b8f4/examples/FinRL_Ensemble_StockTrading_ICAIF_2020.ipynb)

### G179 | FinRL-DeepSeek - LLM-Infused Risk-Sensitive RL
- priority: Medium — Novel LLM-as-risk-controller pattern - cheap to test as an overlay on any daily strategy.
- direction: long | market: equities | timeframe: 1d (daily-native)
- indicators: Price/tech state + LLM-derived risk-sensitivity adjustment
- Overview: Paper implementation: a DeepSeek LLM reads market context and adjusts the RL agent's risk aversion (CVaR-style), making the agent de-risk in regimes the language model flags. The transferable idea: an LLM as a meta-controller modulating position risk rather than predicting price.
- Stock or ETF Criteria: Equities.
- Stocks to avoid and known failure conditions: LLM risk calls are slow/costly; the paper is recent (2025) - evidence thin.
- Entry conditions: RL policy scaled by LLM-derived risk-sensitivity parameter.
- Exit conditions: Per RL policy.
- Trade management: Risk-sensitive reward/objective.
- Target: Better tail-risk-adjusted returns.
- Risk factors and risk management: LLM hallucination risk on regime calls.
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- external_refs: FinRL-DeepSeek arXiv:2502.07393
- notes: Idea: LLM risk-dial overlay - adaptable to non-RL strategies.
- sources: [benstaf/FinRL_DeepSeek — paper code](https://github.com/benstaf/FinRL_DeepSeek/blob/5c21a923214bca6370800efd45f8c6c1ef776ae7/README.md)

### G180 | Deep RL Trading Agent Zoo (grouped: DQN/A2C/PPO/DDQN/A3C across crypto+equity)
- priority: Low — One approach class; thin demonstrated OOS edge - keep FinRL + DeepSeek as the extraction targets.
- direction: long/short | market: crypto + equities | timeframe: 1m-1d (daily-native)
- indicators: OHLCV state windows -> discrete buy/hold/sell or weight actions; gym-style envs
- Overview: Grouped entry for the large RL-agent repo family: tensortrade, TradeMaster, Yvictor/TradingGym, notadamking/RLTrader, deependersingla/deep_trader, Rachnog/Deep-Trading, Ceruleanacg/Personae, sadighian/crypto-rl, samre12/deep-trading-agent, hackthemarket/gym-trading, Albert-Z-Guo/Deep-Reinforcement-Stock-Trading, thedimlebowski/Trading-Gym, ucaiado (both), evgps/a3c_trading, TorchTrade, pythonlessons/RL-Bitcoin, golsun/deep-RL-trading, yuriak/RLQuant, sachink2010, javifalces, ucaiado/rl_trading, hugocen/freqtrade-gym, LiamConnell, shivamakhauri04, Amey-Thakur, hahnicity/pytrader, LastAncientOne, huseinzol05, Packt x2, kknet, SravB, theanh97, jcwill415, xljiang, lnmangione, jsgaston, Anshupriya2694, sushantsbelapurkar. All implement gym-style agents on candles; the honest assessment: none demonstrates robust OOS edge - treat as one approach class (RL policy on bars) with FinRL as the reference implementation and the ensemble/turbulence variant as the standout.
- Stock or ETF Criteria: Gym-style OHLCV envs; crypto or equities.
- Stocks to avoid and known failure conditions: RL overfitting is universal here; most report train-set 'profits'; few honest cost models.
- Entry conditions: Agent action per bar.
- Exit conditions: Agent action.
- Trade management: Per env.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): 1m-1d per repo.
- reported_metrics: none reported
- notes: Deduplicated RL family; FinRL/FinRL-DeepSeek are the standouts.
- sources: [tensortrade-org/tensortrade — RL framework (+2 more verified paths)](https://github.com/tensortrade-org/tensortrade/blob/d58afba23deb1fded39793203b7997c3990bb032/README.md); [tensortrade-org/tensortrade — tensortrade](https://github.com/tensortrade-org/tensortrade/blob/d58afba23deb1fded39793203b7997c3990bb032/tensortrade); [tensortrade-org/tensortrade — examples](https://github.com/tensortrade-org/tensortrade/blob/d58afba23deb1fded39793203b7997c3990bb032/examples); [TradeMaster-NTU/TradeMaster — RL platform (+1 more verified paths)](https://github.com/TradeMaster-NTU/TradeMaster/blob/1747cc18db3fe2639af12defc80e138c51a625c0/README.md); [TradeMaster-NTU/TradeMaster — trademaster](https://github.com/TradeMaster-NTU/TradeMaster/blob/1747cc18db3fe2639af12defc80e138c51a625c0/trademaster); [AI4Finance-Foundation/FinRL — reference impl (+1 more verified paths)](https://github.com/AI4Finance-Foundation/FinRL/blob/7e98df461cbc7772f16a545a2f13b521e5d0b8f4/finrl/agents/); [AI4Finance-Foundation/FinRL — examples/](https://github.com/AI4Finance-Foundation/FinRL/blob/7e98df461cbc7772f16a545a2f13b521e5d0b8f4/examples/)

### G181 | Deep Reinforcement Learning Pair-Position Policy
- priority: Low — The learned action policy is a distinct RL approach, but validation and test filtering still create selection risk and no independent OOS evidence is reported.
- direction: long/short | market: Equities; pair universe drawn from listed stocks | timeframe: Daily prices (daily-native)
- indicators: Normalized log prices, pair spread, intercept/hedge ratio state; discrete flat/long/short actions
- Overview: The notebook trains an agent to choose flat, long-spread, or short-spread actions across thousands of precomputed pair slices. Reward is change in pair portfolio value with commissions in the cost-enabled variant.
- Stock or ETF Criteria: Precomputed pair slices with price/spread features; notebook enumerates up to 6,670 pairs.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: At each environment step choose action 0=flat, 1=long spread (long Y/short X), or 2=short spread (short Y/long X); the agent learns the policy from the normalized pair history and current position.
- Exit conditions: Action 0 exits any open pair; switching directly between long and short closes/reopens the spread. The environment forces flat if the short-side value becomes nonpositive.
- Trade management: Action-controlled pair positions; the cost-enabled notebook includes commission and separates train/validation/test pair slices.
- Target: Maximize discounted portfolio reward; no fixed z-score exit.
- Risk factors and risk management: The agent ranks pairs by training reward and filters by validation reward; pair selection can overfit and exact network/training hyperparameters are not specified in the source excerpts.
- Best known candlestick timeframe(s): Daily pair data are used.
- reported_metrics: none reported
- notes: The uncommissioned model/trading_model.ipynb is not used as a separate strategy; the commission-enabled environment is the cited version.
- sources: [wywongbd/pairstrade-fyp-2019 — Discrete pair actions, portfolio reward, and forced short-risk exit](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/model/trading_env.py#L146-L347); [wywongbd/pairstrade-fyp-2019 — Pair-slice training and validation/test evaluation; code cells 3–12 and 20](https://github.com/wywongbd/pairstrade-fyp-2019/blob/078c84b6ad0b941a0ccd497e4a803cbec78f7165/model/trading_model_with_comission.ipynb)

### G182 | LSTM Policy-Gradient Pair-Trading Agent
- priority: Low — The sequential decision approach is distinct but lacks costs, validation evidence, and realistic risk controls.
- direction: long/short | market: Equity pair in the simulator; exact symbols vary | timeframe: Daily prices (daily-native)
- indicators: Sequence of market-state features, 20-unit LSTM, three actions (buy/sell/hold), portfolio-value reward
- Overview: An LSTM policy-gradient agent selects buy, sell, or hold actions for a two-leg equity portfolio and receives reward from changes in portfolio value. It is a learned action policy rather than a fixed z-score strategy.
- Stock or ETF Criteria: Two equity legs and daily market-state features; source examples use a historical simulator.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Initialize the policy with a 50-observation market-state lookback; the LSTM produces a three-action softmax policy. The environment interprets buy as long A/short B and sell as short A/long B (or closes/reverses an existing position).
- Exit conditions: The agent may hold until a subsequent buy/sell action; the environment has no fixed stop, target, or time exit.
- Trade management: Fixed 100-share lots per leg in the environment; no transaction costs or risk limits are specified.
- Target: Reward maximization through sequential buy/sell/hold actions.
- Risk factors and risk management: The repository uses 2008–2009-style sample data, has no costs, and does not document a chronological OOS protocol.
- Best known candlestick timeframe(s): Daily data are used; learned state length defaults to 50 observations.
- reported_metrics: none reported
- notes: The environment implements a two-stock pair simulator; no recommendation or live trading path is inferred.
- sources: [shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading — LSTM policy network, reward-weighted training and policy-gradient agent](https://github.com/shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading/blob/e377346fad5d8e22c2ae2d61b893c0033da04caf/RLMDP/agent.py#L7-L224); [shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading — 50-observation state, three actions and pair portfolio transitions](https://github.com/shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading/blob/e377346fad5d8e22c2ae2d61b893c0033da04caf/RLMDP/environment.py#L73-L224)


## Crypto Multi-Indicator Bot Strategies

### G183 | BBGO Strategy Suite - Supertrend, TrendTrader, PivotShort, FlashCrash, EWO, FactorZoo, Drift
- priority: Medium — Real strategy variety beyond indicator soup; EWO divergence and flashcrash-buy are distinctive testable mechanisms.
- direction: long/short | market: crypto multi-exchange (Go) | timeframe: configurable candles (intraday-adaptable)
- indicators: supertrend (ATR), trendtrader, pivotshort (pivot short reversion), flashcrash (crash-buy), ewoDgtrd (Elliott-wave-osc divergence), factorzoo (multi-factor), drift, emacross, elliottwave, harmonic, atrpin, swing
- Overview: 40+ built-in Go strategies; the directional/non-MM subset worth extraction: supertrend (ATR-band trend following), trendtrader (trend-following composite), pivotshort (short at pivot resistance), flashcrash (buy flash crashes), ewoDgtrd (Elliott-wave oscillator divergence), factorzoo (multi-factor scoring), drift (drift reversion), emacross, elliottwave, harmonic patterns, swing. Several are unusual mechanisms absent from the catalog (EWO divergence, flashcrash buying).
- Stock or ETF Criteria: Exchange-supported crypto pairs.
- Stocks to avoid and known failure conditions: Mixed maturity - some are examples; grid/MM entries excluded per scope.
- Entry conditions: Per-strategy: supertrend direction flip, pivotshort at resistance, flashcrash on crash-detection, ewo divergence.
- Exit conditions: Per-strategy signals/stops.
- Trade management: YAML-configured per strategy.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Configurable; most run on 1h-1d.
- reported_metrics: none reported
- notes: Market-making/funding/grid strategies excluded per scope.
- sources: [c9s/bbgo — supertrend](https://github.com/c9s/bbgo/blob/2807e176558e6d8c5ba9c990f3488243f3e5fbfb/pkg/strategy/supertrend); [c9s/bbgo — pivot short](https://github.com/c9s/bbgo/blob/2807e176558e6d8c5ba9c990f3488243f3e5fbfb/pkg/strategy/pivotshort); [c9s/bbgo — flash crash buy](https://github.com/c9s/bbgo/blob/2807e176558e6d8c5ba9c990f3488243f3e5fbfb/pkg/strategy/flashcrash); [c9s/bbgo — EWO divergence](https://github.com/c9s/bbgo/blob/2807e176558e6d8c5ba9c990f3488243f3e5fbfb/pkg/strategy/ewoDgtrd); [c9s/bbgo — factor zoo](https://github.com/c9s/bbgo/blob/2807e176558e6d8c5ba9c990f3488243f3e5fbfb/pkg/strategy/factorzoo)

### G184 | Freqtrade Community Strategy Collection - Indicator Combo Families
- priority: Medium — High-volume reference implementations; a handful (Cluc, BbandRsi, HourBased) worth direct testing, the rest are parameter variants.
- direction: long | market: crypto (freqtrade) | timeframe: 5m-1h typical (intraday-adaptable)
- indicators: RSI, Bollinger, EMA/TEMA crosses, MACD, Stoch, ADX, MFI, SAR, CCI, harami candle patterns
- Overview: ~50 community IStrategy implementations - mostly indicator-combination dip-buyers and trend-followers. Distinct members: ClucMay72018 (BB inside-candle dip-buy), BbandRsi (BB lower + RSI), Quickie (MACD+MA+RSI fast scalper), Strategy001-005 (RSI+EMA multi-condition buys), HourBasedStrategy (hour-of-day gated), TrendRiderStrategy (trend-riding exits), AlmgrenChrissStrategy (execution scheduling). Groups into: (a) BB/RSI dip-buyers, (b) EMA-cross trend followers, (c) multi-confirmation composites.
- Stock or ETF Criteria: Freqtrade pairlists (VolumePairList typical).
- Stocks to avoid and known failure conditions: Most are hyperopt-era indicator soups - expect parameter fragility; 5m crypto signals don't transfer to daily.
- Entry conditions: Per-strategy populate_buy_trend: e.g., ClucMay72018: close < BB_lower AND candle fully inside BB AND volume>0; BbandRsi: close<BB_lower AND RSI<30; Quickie: MACD hist>0 AND MA up AND RSI improving.
- Exit conditions: minimal_roi tables + stoploss + sell_signal RSI/MACD exits per strategy.
- Trade management: Freqtrade stake + optional DCA.
- Target: Small ROI targets (0.5-5%).
- Risk factors and risk management: Standard freqtrade stoploss/ROI; few have real risk models.
- Best known candlestick timeframe(s): 5m-1h native. The BB-dip and RSI-oversold cores map to daily S011/S001 family.
- reported_metrics: none reported
- notes: Deduped: near-identical RSI/BB/EMA variants collapsed into this grouped entry.
- sources: [freqtrade/freqtrade-strategies — ~50 IStrategy files: ClucMay72018, BbandRsi, Quickie, Strategy001-005, HourBasedStrategy, TrendRiderStrategy, AlmgrenChrissStrategy](https://github.com/freqtrade/freqtrade-strategies/blob/f3340ce11f5bdf62f598522e64d1f5638eaa13f5/user_data/strategies/)

### G185 | Haehnchen Dip-Catcher / DCA-Dipper / OBV Pump-Dump / Pivot-Reversal Set
- priority: Medium — dip_catcher/pivot_reversal/obv_pump_dump are distinct daily-testable mechanics (capitulation, S/R, volume-confirmed reversal).
- direction: long | market: crypto multi-pair (CCXT) | timeframe: intraday candles (intraday-adaptable)
- indicators: dip_catcher + dca_dipper (multi-layer dip buying), obv_pump_dump (OBV volume-spike reversals), pivot_reversal (pivot-point S/R), cci/cci_macd/macd/parabolicsar/awesome_oscillator
- Overview: Multi-strategy crypto bot where the interesting members are the dip strategies: dip_catcher buys sudden price drops (capitulation reversion), dca_dipper layers DCA entries into dips, obv_pump_dump trades OBV-confirmed pump/dump reversals, and pivot_reversal_strategy fades pivot-level touches. Plus the canonical cci/macd/sar/ao strategies.
- Stock or ETF Criteria: Multi-pair via CCXT; talib indicators.
- Stocks to avoid and known failure conditions: Dip-catching is knife-catching without a floor filter; DCA tail risk.
- Entry conditions: dip_catcher: sudden % drop trigger; obv_pump_dump: OBV divergence + price extreme; pivot_reversal: touch of daily pivot S/R; cci/macd/sar per textbook.
- Exit conditions: Per-strategy profit target / signal reversal.
- Trade management: dca_dipper: staged adds; profile-based bot manager.
- Target: Mean reversion post-dip.
- Risk factors and risk management: Layered adds risk; no explicit vol gate.
- Best known candlestick timeframe(s): Intraday; daily pivot-reversal and dip-catch are daily-testable.
- reported_metrics: none reported
- notes: Overlap S029 (capitulation-volume) and S030 (down-streak) vicinity.
- sources: [Haehnchen/crypto-trading-bot — dip trigger](https://github.com/Haehnchen/crypto-trading-bot/blob/cb9563344a20ea7ac89ff214d350f141d1521219/src/strategy/strategies/dip_catcher); [Haehnchen/crypto-trading-bot — staged dip DCA](https://github.com/Haehnchen/crypto-trading-bot/blob/cb9563344a20ea7ac89ff214d350f141d1521219/src/strategy/strategies/dca_dipper); [Haehnchen/crypto-trading-bot — OBV divergence](https://github.com/Haehnchen/crypto-trading-bot/blob/cb9563344a20ea7ac89ff214d350f141d1521219/src/strategy/strategies/obv_pump_dump.ts); [Haehnchen/crypto-trading-bot — pivot S/R](https://github.com/Haehnchen/crypto-trading-bot/blob/cb9563344a20ea7ac89ff214d350f141d1521219/src/strategy/strategies/pivot_reversal_strategy.ts)

### G186 | KAC Index v1 - TOTAL3-Indexed ATR+CCI Altcoin Relative-Strength Entries
- priority: Medium — The indexation-to-relative-strength idea is a real, transferable concept (like stock/sector relative strength), though the repo implementation is infra-fragile and intraday.
- direction: long | market: crypto (altcoins indexed by TOTAL3 market cap) | timeframe: 1h (intraday-adaptable)
- indicators: Price divided by smoothed TOTAL3 market-cap index; ATR + CCI multi-filter on indexed prices
- Overview: Divides each altcoin's price by the TOTAL3 (ex-BTC/ETH) market-cap index, then runs ATR+CCI filters on the indexed series - i.e., trades relative strength vs the altcoin market rather than absolute price. The indexation removes market-wide beta so the signal isolates idiosyncratic moves.
- Stock or ETF Criteria: Altcoin perp pairs with sufficient history vs TOTAL3; requires TradingView TOTAL3 feed via tvdatafeed.
- Stocks to avoid and known failure conditions: tvdatafeed dependency is a single point of failure; -90% stoploss in v1 effectively means no stop - crash risk borne fully.
- Entry conditions: Indexed ATR + CCI multi-filter entry on 1h bars (v1 multi-condition).
- Exit conditions: Signal/exit logic on indexed series; stoploss -90% (v1).
- Trade management: not specified in source. Suggested: size down for perp leverage.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: TOTAL3 feed lag/accuracy directly affects signals; indexed series can stay 'oversold' while absolute price still falls with the market.
- Best known candlestick timeframe(s): 1h native; daily adaptation possible (daily close / TOTAL3) - the relative-value idea transfers better than the parameters.
- reported_metrics: Repo includes analysis docs (repo-claimed, unverified).
- notes: Analogous equity idea: divide stock by sector index, trade indexed ATR/CCI - worth testing as a residual-strength variant.
- sources: [titouannwtt/freqtrade-ultimate — strategy code](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/kac_index_v1.py); [titouannwtt/freqtrade-ultimate — philosophy](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/kac_index_v1_readme.md)

### G187 | KAC Index v2 - TOTAL3-Indexed ATR>2.75 Single-Filter Entries
- priority: Medium — Parsimonious relative-strength breakout; transferable to equity sector-relative vol breakouts on daily bars.
- direction: long | market: crypto (altcoins indexed by TOTAL3) | timeframe: 1h (intraday-adaptable)
- indicators: Smoothed TOTAL3 market-cap indexation; single ATR>2.75 filter on indexed prices
- Overview: Simplified successor to KAC v1: same price/TOTAL3 indexation, but the only entry filter is indexed ATR > 2.75 - deliberately fewer parameters to reduce overfitting, more frequent trades, tighter -40% stop. Trades altcoin volatility expansion in market-cap-relative terms.
- Stock or ETF Criteria: Altcoin perps; TOTAL3 feed via tvdatafeed (24-28h smoothing).
- Stocks to avoid and known failure conditions: Single-filter entry gives more whipsaw in low-vol regimes; TradingView dependency; -40% stop still wide.
- Entry conditions: Indexed ATR > 2.75 on 1h bars.
- Exit conditions: Exit signals + stoploss -40%; hyperopt JSON params override code defaults.
- Trade management: max_open_trades -1; hyperopt-tuned params in kac_index_v2.json.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Same TOTAL3 feed fragility; fewer filters = more false starts.
- Best known candlestick timeframe(s): 1h native; daily adaptation straightforward (indexed daily ATR breakout).
- reported_metrics: Repo-claimed smoother equity/higher Sharpe vs v1 (repo-claimed, unverified).
- notes: Equity analog: ATR breakout on stock/sector-index relative price - candidate for cross-sectional vol-expansion strategy.
- sources: [titouannwtt/freqtrade-ultimate — strategy code](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/kac_index_v2.py); [titouannwtt/freqtrade-ultimate — hyperopt params](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/kac_index_v2.json); [titouannwtt/freqtrade-ultimate — v1->v2 changes](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/kac_index_v2_readme.md)

### G188 | NostalgiaForInfinity (NFI) - Protected Dip-Buying Grid with DCA
- priority: Medium — Famous battle-tested structure; the protection-layered dip-buy design is the transferable asset - exact signals are overfit to crypto microstructure.
- direction: long | market: crypto spot/futures (Binance etc.) | timeframe: 5m (with 1h/1d informative) (intraday-adaptable)
- indicators: Dozens of buy-condition 'signals' each gated by protection layers: EMA/RSI/BB dip detection, pump-protection (recent-return caps), volume filters, BTC-pair trend gates, max-drawdown guards; hyperopt-tuned ROI/DCA/exit params
- Overview: The most-starred freqtrade strategy: a systematic dip-buyer that fires ~60+ alternative entry 'signals', each wrapped in dozens of explicit protections (no buying into pumps, dead-cat bounces, illiquid pairs, or adverse BTC regimes), then manages positions with hyperopt-optimized ROI targets, trailing exits and multi-step DCA. The edge thesis: crypto's violent mean-reverting dips are buyable IF you filter out every known failure mode - the protections ARE the strategy.
- Stock or ETF Criteria: Top-volume spot pairs via pairlist filters (VolumePairList, range filters); excludes stables/fiat/blacklist.
- Stocks to avoid and known failure conditions: Hyperopt-tuned parameters = structural overfit risk (documented by author); requires bull/sideways crypto regime; protections are version-specific to the 2020-2023 market.
- Entry conditions: Any of ~60+ named buy signal families (e.g., normal dips, breakouts, sqzmom, local_uptrend, dead-cat bounces, clucHA, nfix_*) each with per-signal enable flags and ~40 global protections: EMA downtrend filters, RSI bounds, recent-range pump caps, volume spikes, BTC/stake currency filters.
- Exit conditions: Per-signal ROI tables + custom_exit logic; stoploss configurable; DCA (max_safety_orders) for averaging down.
- Trade management: Hyperopt-tuned stake/DCA ladder; protections gate ALL layers.
- Target: Multi-tier ROI table (small quick gains); trailing on profit.
- Risk factors and risk management: Layered protections = risk engine; pairlist volume filters; stake/balance ratio caps.
- Best known candlestick timeframe(s): 5m entries with 1h/1d informative trend. Daily adaptation: keep the protection concept (dip-entry gated by regime/volume/pump filters) - the exact params won't transfer.
- reported_metrics: Community-verified live track record exists but no canonical numbers (repo-claimed, unverified).
- notes: Signals are toggled by config; treat the protection library, not the entry catalog, as the design lesson.
- sources: [iterativv/NostalgiaForInfinity — signal families + protection layers](https://github.com/iterativv/NostalgiaForInfinity/blob/5bbcf197a2ec013341974ff70d2047d1b303dcd3/user_data/strategies/NostalgiaForInfinityX8.py); [iterativv/NostalgiaForInfinity — per-signal guard tests](https://github.com/iterativv/NostalgiaForInfinity/blob/5bbcf197a2ec013341974ff70d2047d1b303dcd3/tests/unit/)

### G189 | Simple VWAP v1 - Constant-Exposure VWAP-Band Mean Reversion with DCA
- priority: Medium — Distinct always-in-the-market design with documented exits and real backtest artifacts; 4h-native keeps it off the daily-bars focus.
- direction: long | market: crypto perps (Hyperliquid/Binance futures) | timeframe: 4h (4h-plus)
- indicators: VWAP + rolling bands (entry anchor), EMA trend-reversal exit, CCI momentum-decay exit
- Overview: Contrarian freqtrade strategy that stays ~90% invested: buys each touch of the VWAP lower band after an 8-candle confirmation, trusting regression to the VWAP mean, and manages risk through exits plus up to 4 exponentially-spaced DCA safety orders rather than tight stops. Edge thesis: continuous deployment captures mean-reverting chop that 'sniper' strategies wait out.
- Stock or ETF Criteria: Liquid crypto perp pairs; up to 40 simultaneous positions, so a diversified pairlist is required.
- Stocks to avoid and known failure conditions: Designed for sideways/chop; underperforms strong trends. -37% stoploss plus DCA needs capital headroom; concentrated pair risk if pairlist is thin.
- Entry conditions: VWAP lower-band touch + 8-candle confirmation on 4h bars.
- Exit conditions: EMA trend reversal OR negative CCI momentum; hard stoploss -37% (rarely hit).
- Trade management: Dynamic volatility-adjusted sizing; up to 4 DCA safety orders with exponential spacing; max 40 open trades; tradable_balance_ratio 0.99.
- Target: Small frequent gains from VWAP reversion (no fixed TP; exit-signal driven).
- Risk factors and risk management: Wide stop (-37%) makes sizing critical; DCA averaging-down tail risk in crashes.
- Best known candlestick timeframe(s): Native 4h. Daily adaptation plausible (VWAP->anchored mean on daily closes) but the high-frequency small-gain economics depend on sub-daily fills.
- reported_metrics: Repo includes backtest/walk-forward/PBO analysis in simple_vwap_v1_analysis.md (repo-claimed, unverified).
- notes: Fork is Hyperliquid-tuned freqtrade; liquidation detection and shared-wallet guards are platform features, not edge.
- sources: [titouannwtt/freqtrade-ultimate — strategy code](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/simple_vwap_v1.py); [titouannwtt/freqtrade-ultimate — setup params + honest assessment](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/simple_vwap_v1_readme.md); [titouannwtt/freqtrade-ultimate — backtest analysis](https://github.com/titouannwtt/freqtrade-ultimate/blob/878206c5a3be95d269ed2ff9418071df059171a4/user_data/strategies/simple_vwap_v1_analysis.md)

### G190 | Binance Overbought-Composite Short (RSI70 + BB mid + MACD + EMA20 filter)
- priority: Low — Contradictory indicator soup with infra quirks; the short-side overbought composite is the only transferable idea.
- direction: short | market: crypto spot (Binance, e.g. BNB/USDT) | timeframe: intraday (poll loop) (intraday-adaptable)
- indicators: SMA30, EMA20, MACD + signal, Bollinger bands (mid/upper), RSI14
- Overview: Short-only mean-reversion bot: shorts when price is stretched above its trend context (above SMA30, above BB middle, below upper band, MACD above signal, RSI>70) yet below EMA20 - i.e., an overbought stall. Places a limit sell and a paired profit-target limit buy with an order-timeout fallback to market.
- Stock or ETF Criteria: Liquid Binance spot pair (symbol config, e.g. BNB/USDT).
- Stocks to avoid and known failure conditions: The entry conjunction is contradictory (price>SMA30 and >BB-mid but <EMA20) - rare, fragile signals; shorting spot on Binance is borrow-limited anyway.
- Entry conditions: All of: price>SMA30, RSI>70, price>BB-middle and <BB-upper, MACD>signal, price<EMA20 -> limit short.
- Exit conditions: Paired limit buy at profit_target; order_timeout cancels and market-closes whichever leg filled.
- Trade management: Fixed volume_threshold per trade; timeout-based order reconciliation.
- Target: Fixed profit_target offset below entry (customizable).
- Risk factors and risk management: No stop on the entry itself beyond the timeout mechanism; short squeeze tail risk.
- Best known candlestick timeframe(s): Poll-loop spot bot; bar length implicit. Daily adaptation loses the paired-limit microstructure.
- reported_metrics: none reported
- notes: Entry_price/profit_target are '...' placeholders - parameters left to user.
- sources: [guldo111/binance-mean-reversion-trading-bot — mean_reversion_strategy entry conjunction + paired order logic](https://github.com/guldo111/binance-mean-reversion-trading-bot/blob/53dc2ddd1c348ee533d4c0d99cc8056b6e28c7bd/bot.py)

### G191 | Crypto-Signal Alert Strategy Set (RSI/MACD/Ichimoku/OBV/Momentum/Breakout)
- priority: Low — Canonical signal set as a scanner; individual signals are textbook.
- direction: long | market: crypto (multi-exchange via CCXT) | timeframe: 1h-1d configurable (intraday-adaptable)
- indicators: RSI, MACD, Ichimoku, OBV, momentum, moving-average cross, breakout/volume alerts
- Overview: Alert-bot that computes a fixed menu of canonical signals across many pairs: RSI thresholds, MACD cross, Ichimoku cloud state, OBV divergence/pumps, momentum and breakout alerts, then dispatches hot/cold alerts. Each indicator is a tradeable signal family; the value is the multi-pair scanning pattern.
- Stock or ETF Criteria: Liquid CCXT pairs with OHLCV.
- Stocks to avoid and known failure conditions: Alert bot (no execution) - slippage ignored; signal quality per indicator is textbook.
- Entry conditions: Per-indicator thresholds (e.g., RSI<30 oversold alert, MACD cross, ichimoku cloud break, OBV divergence).
- Exit conditions: Alert-only (no position management).
- Trade management: N/A.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): 1h-1d candles.
- reported_metrics: none reported
- notes: Overlap with Gekko set; keep as scanning-pattern reference.
- sources: [CryptoSignal/Crypto-Signal — indicator alert behaviors](https://github.com/CryptoSignal/Crypto-Signal/blob/7cb9c5c6cd226c6fe2d345e4bda3bec8156cefec/app/behaviour.py); [CryptoSignal/Crypto-Signal — signal docs](https://github.com/CryptoSignal/Crypto-Signal/blob/7cb9c5c6cd226c6fe2d345e4bda3bec8156cefec/docs/)

### G192 | Gekko Classic Indicator Strategy Set (RSI/MACD/DEMA/PPO/StochRSI/TSI/UO/CCI/ADX)
- priority: Low — Reference baselines, not novel edges - value is as clean control strategies.
- direction: long | market: crypto (Gekko bot) | timeframe: 1h typical (intraday-adaptable)
- indicators: Canonical single/dual-indicator triggers: RSI thresholds, MACD hist, DEMA, PPO, StochRSI, TSI, Ultimate Oscillator, CCI, tulip ADX
- Overview: Gekko ships the reference implementations of ~15 canonical indicator strategies (one trigger family each): RSI oversold/overbought, MACD histogram crosses, DEMA/PPO/TSI momentum, StochRSI extremes, CCI, UO, ADX trend-strength. Useful as clean minimal baselines - each is the textbook definition of its signal, ideal control strategies for the partition test.
- Stock or ETF Criteria: Any crypto pair.
- Stocks to avoid and known failure conditions: Single-indicator strategies are whipsaw-prone; meant as building blocks.
- Entry conditions: Per strategy: e.g., RSI: buy when RSI < low threshold after being oversold; MACD: buy on MACD crossing above signal.
- Exit conditions: Opposite threshold / signal flip.
- Trade management: One position per market.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): 1h+ candles; daily versions = catalog oscillator families.
- reported_metrics: none reported
- notes: Overlap S001/S011/S088 vicinity; keep as baseline controls.
- sources: [askmike/gekko — RSI.js MACD.js DEMA.js PPO.js StochRSI.js TSI.js UO.js CCI-related tulip-adx.js](https://github.com/askmike/gekko/blob/ff5698f1ae6329bab5729f145c35a74161e82bac/strategies/)

### G193 | Gekko-Strategies Community Set with Backtest Reports
- priority: Low — Tuned indicator soups; useful for parameter ranges, not new mechanisms.
- direction: long | market: crypto (Gekko) | timeframe: 1h-1d (intraday-adaptable)
- indicators: Mixed: BitBank variants, stratego_sma versions, multi-indicator combos with tuned TOML configs
- Overview: Large community collection of Gekko strategies with TOML configs and some backtest results. Notable: BitBankStrategy family (multi-iteration tuned), stratego_smaxv7 with stop-loss, various indicator-soup composites with reported per-pair results.
- Stock or ETF Criteria: Gekko backtestable pairs.
- Stocks to avoid and known failure conditions: Heavily tuned configs per pair = overfit; Gekko is archived.
- Entry conditions: Per-strategy JS check() logic; tuned thresholds in configs/*.toml.
- Exit conditions: Per-strategy; some have explicit stop-loss params.
- Trade management: Per-config.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Tuned-per-pair configs = selection bias.
- Best known candlestick timeframe(s): 1h-1d candles per config.
- reported_metrics: Repo includes per-pair backtest results in some configs (repo-claimed, unverified).
- notes: Deduped into one entry; configs hold the tuned params.
- sources: [xFFFFF/Gekko-Strategies — BitBank + stratego_sma variants and configs](https://github.com/xFFFFF/Gekko-Strategies/blob/f53854f8555a7c962f4013c53c464cfb377f83f6/%21WORKSOP/)

### G194 | Jesse Framework Strategy Stubs (test/regression strategies)
- priority: Low — Framework, not alpha - noted for the clean strategy interface.
- direction: long/short | market: crypto (Jesse) | timeframe: configurable (intraday-adaptable)
- indicators: Per-strategy; framework provides indicator library
- Overview: Jesse is a framework; bundled strategies are test/regression stubs rather than production logic. Documented: the strategy model (should_long/should_short + go_long/go_short) is the clean contract for porting daily-bar strategies.
- Stock or ETF Criteria: Crypto pairs.
- Stocks to avoid and known failure conditions: No production strategies in-repo.
- Entry conditions: should_long()/should_short() contract.
- Exit conditions: update_position/go_short contract.
- Trade management: Framework-managed.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Configurable.
- reported_metrics: none reported
- notes: Framework-level entry.
- sources: [jesse-ai/jesse — strategy contract](https://github.com/jesse-ai/jesse/blob/4292249a2c478b1c0eb64ce426ddada2207e19b2/jesse/strategies/Strategy.py); [jesse-ai/jesse — example stubs](https://github.com/jesse-ai/jesse/blob/4292249a2c478b1c0eb64ce426ddada2207e19b2/jesse/strategies/)

### G195 | Ninjabot Example Strategies (Go framework examples)
- priority: Low — Framework examples.
- direction: long/short | market: crypto (Binance) | timeframe: configurable (intraday-adaptable)
- indicators: Example strategies in Go
- Overview: Fast Go crypto bot; strategies are examples demonstrating the Strategy interface rather than production edges. Noted for completeness.
- Stock or ETF Criteria: Binance pairs.
- Stocks to avoid and known failure conditions: Example-grade.
- Entry conditions: Per example strategy.
- Exit conditions: Per example.
- Trade management: not specified in source. Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Configurable.
- reported_metrics: none reported
- notes: Framework examples.
- sources: [rodrigo-brito/ninjabot — example strategies](https://github.com/rodrigo-brito/ninjabot/blob/ff61da5f3d0c0b03a743ab0ddb05a382593caf73/examples/strategies/)

### G196 | PyCryptoBot EMA12/26 Golden-Cross + MACD Confirmation Bot
- priority: Low — Two-confirmation EMA/MACD cross is textbook; the no-chase exclusion is the only novel bit.
- direction: long | market: crypto (Coinbase/Binance) | timeframe: 1h-1d configurable (intraday-adaptable)
- indicators: EMA12/EMA26 crossover, MACD>signal, 3%-of-close-high buy exclusion, optional OBV
- Overview: Default bot logic: buy when EMA12 crosses above EMA26 AND MACD is above its signal line (both independently disableable), with a buy-exclusion if price is within 3% of the dataframe high (don't chase tops). A clean two-confirmation trend-entry bot with a no-chase guard.
- Stock or ETF Criteria: Liquid crypto pairs; granularities 1h-1d.
- Stocks to avoid and known failure conditions: Golden-cross lag in chop; the 3%-of-high exclusion blocks entries in strong trends (by design).
- Entry conditions: ema12gtema26co AND macdgtsignal AND NOT within 3% of close-high.
- Exit conditions: EMA cross down / MACD cross down; smart-switch and configurable sell triggers.
- Trade management: Position-size %, sell-at-loss toggles, margin calc.
- Target: Trend continuation post-cross.
- Risk factors and risk management: Buy-exclusion guards against top-chasing; per-market settings.
- Best known candlestick timeframe(s): 1h-1d.
- reported_metrics: none reported
- notes: Overlap S088/S089 vicinity.
- sources: [whittlem/pycryptobot — is_buy_signal logic](https://github.com/whittlem/pycryptobot/blob/1fa9aaef141725623899230daa585e17fa7ba007/models/Strategy.py); [whittlem/pycryptobot — custom example](https://github.com/whittlem/pycryptobot/blob/1fa9aaef141725623899230daa585e17fa7ba007/examples/chatgpt/bbands_fib_strategy.py)


## Prediction-Market Strategies

### G197 | Prediction-Market Strategy Families (convergence, momentum, copy-trade, last-seconds, dump-hedge)
- priority: Low — Niche venue edge; copy-trade/flow-signal concept is the only transferable idea (whale tracking = informed-flow).
- direction: long/short | market: Polymarket/Kalshi/Manifold | timeframe: event-driven (intraday-adaptable)
- indicators: Event-contract pricing, odds momentum, whale-wallet copy signals, market convergence (yes+no sum), late-window pricing
- Overview: Grouped entry for the prediction-market strategy family across the 6 repos: polytutor's strategy guides (copy-trading whales, dump-hedge, last-seconds, sports), OctoBot-PM copy-trading, kalshi bots (momentum/reversion), emmanuelwestra's 7-strategy bot (arbitrage, convergence, market-making, momentum, AI-forecast, whale tracker). The transferable patterns: odds-momentum and convergence to 1 (yes+no should sum to $1) are event-market-native; whale-copy-trading is an alternative-data flow signal.
- Stock or ETF Criteria: Prediction-market contracts with liquidity.
- Stocks to avoid and known failure conditions: Prediction-market edges don't transfer to equities except copy-trade/flow concepts; many venues are thin.
- Entry conditions: Momentum/reversion on contract prices; convergence when yes+no != $1; copy whale wallets; last-seconds pricing anomalies.
- Exit conditions: Event resolution or signal decay.
- Trade management: Per-strategy.
- Target: Contract repricing.
- Risk factors and risk management: Illiquidity; resolution risk.
- Best known candlestick timeframe(s): Intraday to event horizon.
- reported_metrics: none reported
- notes: Grouped prediction-market family.
- sources: [Polytutor-AI-Labs/polymarket-trading-bot-strategy — guides (+2 more verified paths)](https://github.com/Polytutor-AI-Labs/polymarket-trading-bot-strategy/blob/c4b7f872cefa65b1c3b5a2ae960af0a9ed3ff48a/README.md); [Polytutor-AI-Labs/polymarket-trading-bot-strategy — docs](https://github.com/Polytutor-AI-Labs/polymarket-trading-bot-strategy/blob/c4b7f872cefa65b1c3b5a2ae960af0a9ed3ff48a/docs); [Polytutor-AI-Labs/polymarket-trading-bot-strategy — strategies](https://github.com/Polytutor-AI-Labs/polymarket-trading-bot-strategy/blob/c4b7f872cefa65b1c3b5a2ae960af0a9ed3ff48a/strategies); [emmanuelwestra/Polymarket-Trading-Bot — 7 strategies (+1 more verified paths)](https://github.com/emmanuelwestra/Polymarket-Trading-Bot/blob/54e80a6b6ac6551db458380e14d10b14d777ce3c/README.md); [emmanuelwestra/Polymarket-Trading-Bot — src](https://github.com/emmanuelwestra/Polymarket-Trading-Bot/blob/54e80a6b6ac6551db458380e14d10b14d777ce3c/src); [lalouannabel0072/kalshi-trading-bot — kalshi](https://github.com/lalouannabel0072/kalshi-trading-bot/blob/007d78c85fa9d93e45e63907deb17fb2e6b4a506/README.md); [else24/kalshi-market-bot — kalshi](https://github.com/else24/kalshi-market-bot/blob/f09382fac1a461c82806b933ea559c2c5f715c4b/README.md); [Drakkar-Software/OctoBot-Prediction-Market — copy trading](https://github.com/Drakkar-Software/OctoBot-Prediction-Market/blob/08e2c835578e536a2e2228da67352ca3405177d9/README.md); [evan-kolberg/prediction-market-backtesting — nautilus ext](https://github.com/evan-kolberg/prediction-market-backtesting/blob/c76e77af00ef53472a9da8f66dae7fdd2d3e5928/README.md)


## Execution-Aware and Miscellaneous

### G198 | Direction-Agnostic Wide-Stop Ratchet-Exit Framework (mr-scrooge v6)
- priority: Medium — Not an alpha but a rigorous falsification + exit-engineering reference; the ratchet/lock design and honest cost accounting are worth reusing on top of real entries.
- direction: long/short | market: FX (OANDA) | timeframe: intraday sessions (intraday-adaptable)
- indicators: Entry setups (multiple cells); exit = wide server-side stops 40/50/60p + two-phase ratchet (engage +5 lock +4, then step peak-2); 'popper' adverse-grid adds at 25/35/45/55p; 4h reaper time-stop
- Overview: The most honest execution research in the inventory: after -84% on predictive entries (0/144 feature combos carried direction), the author concluded entries predict WHEN/HOW FAR price moves, never WHICH WAY - so v6 treats entries as near-arbitrary and makes the EXIT the strategy: wide session-scaled stops with a two-phase ratchet that locks +4 at +5 then trails peak-2, plus same-direction 'popper' adds at adverse 25-55p intervals, all promoted/demoted by a governor on cycle-level live evidence. Documented falsification result: wide-stop walk-forward gross Sharpe 1.26 -> NET 0.03 after costs - the edge equals the toll.
- Stock or ETF Criteria: FX majors on OANDA; session-level pip ranges.
- Stocks to avoid and known failure conditions: The author's own pre-registered result: net ~0 after costs - the wide-stop ratchet converts to breakeven; many-small-wins/few-large-losses asymmetry is fragile to gap-throughs past the stop.
- Entry conditions: Cell-defined session setups (published as cells); direction is de-emphasized - framework works on either side.
- Exit conditions: Stop 40/50/60p by session; ratchet: at +5 lock +4, then each +2 step locks peak-2; no TP, no timeout except 4h reaper on still-red trades; poppers inherit parent's ratchet.
- Trade management: 15% risk per trade, max 6 concurrent (4 for grid eligibility); popper adds retire with parent; governor promotes/demotes cells on >=85% cycle-win + positive pips over 5+ cycles.
- Target: Open-ended via ratchet (no TP).
- Risk factors and risk management: The framework IS the risk system; documented live forward-test record (100-trade practice +10.54%, then live).
- Best known candlestick timeframe(s): Intraday sessions; the ratchet/exit machinery translates to daily bars directly.
- reported_metrics: Pre-registered: 100-trade practice forward test +10.54%; wide-stop h6 walk-forward gross Sharpe 1.26, net 0.03 after costs; live $2.5k record public (repo-claimed but with broker-verified trade logs).
- notes: Negative-result value: documents that wide-stop ratchet exits on directionless entries net ~0 - use its exit machinery, not its entries.
- sources: [BrockStar3540/mr-scrooge-v6 — two-phase ratchet, poppers, governor, track record](https://github.com/BrockStar3540/mr-scrooge-v6/blob/340fdaf263ad28047e6668f7585f90505a3109fe/README.md); [BrockStar3540/mr-scrooge-v6 — pre-registered forward test](https://github.com/BrockStar3540/mr-scrooge-v6/blob/340fdaf263ad28047e6668f7585f90505a3109fe/docs/FORWARD_TEST_100_REPORT.md); [BrockStar3540/mr-scrooge-v6 — live trades](https://github.com/BrockStar3540/mr-scrooge-v6/blob/340fdaf263ad28047e6668f7585f90505a3109fe/livelog/trades.csv)

### G199 | Bid-Ask Depth Imbalance Forecast
- priority: Low — The signal is structurally plausible but requires order-book data and has substantial execution risk.
- direction: long/short | market: US equities | timeframe: Daily and intraday bars (intraday-adaptable)
- indicators: Bid/ask displayed size imbalance normalized by spread
- Overview: The model treats bid/ask depth asymmetry as a predictive signal, scales the imbalance by spread, and fits coefficients to forward returns. One variant adds beta-adjusted returns and six time-of-day coefficient buckets.
- Stock or ETF Criteria: US equities with quote-depth, spread, price and industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Signal=(ask_size−bid_size)/(ask_size+bid_size), normalized by sqrt(spread) and scaled by 1/10000; winsorize and industry-demean. `bsz.py` combines daily/intraday and market-adjusted returns; `bsz1.py` uses a simpler non-beta-adjusted fit.
- Exit conditions: Forecast positions are updated through qsim; explicit price exits are not specified.
- Trade management: Portfolio optimizer/simulator handles order sizing; no standalone stop/target.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday quotes are needed for the full model; daily version depends on available order-book aggregates.
- parameter_variants: bsz.py uses beta-adjusted returns and hourly coefficients; bsz1.py omits beta adjustment and has simpler coefficients.
- reported_metrics: none reported
- notes: Not an indicator-only library: the cited modules fit signals against forward returns and emit alpha forecasts.
- sources: [noterminusgit/statarb — Daily/intraday bid-ask size signal and time-of-day regression](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/bsz.py#L63-L177); [noterminusgit/statarb — Simplified bid-ask size variant](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/bsz1.py#L42-L151)

### G200 | Daily/Intraday Beta-Adjusted Order-Flow Imbalance
- priority: Low — Order flow has a plausible microstructure edge, but it is data-intensive, high-turnover, and not a daily-only swing strategy.
- direction: long/short | market: US equities | timeframe: Daily and 30-minute bars (intraday-adaptable)
- indicators: Aggressive ask/bid hit dollars, mid-hit dollars, spread, beta-adjusted returns
- Overview: The signal combines absolute order-flow imbalance with daily lags and intraday flow, normalizing by square-root spread and demeaning within industry. Positive ask-hit imbalance is treated as buying pressure, with regression coefficients translating it to a forecast.
- Stock or ETF Criteria: US equities with intraday order-flow and spread data plus daily price/industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute bd0=(askHitDollars−bidHitDollars)/(askHitDollars+midHitDollars+bidHitDollars), divide by sqrt(spread_bps), winsorize and industry-demean; apply fitted daily-lag and six hourly intraday coefficients.
- Exit conditions: Forecast is updated daily/intraday; order-level exits and stops are not specified in the alpha module.
- Trade management: Alpha feeds the qsim/order-level simulator; trade execution and position limits are separate.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily component can be modeled daily; intraday order-flow component requires intraday data.
- parameter_variants: bd.py combines daily lags and spread-normalized intraday flow; bd_intra.py is intraday-only without spread normalization.
- reported_metrics: none reported
- notes: The source documents regression fitting before middate; costs and market impact are handled outside these signal modules.
- sources: [noterminusgit/statarb — Order-flow imbalance construction and daily/intraday forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/bd.py#L181-L504); [noterminusgit/statarb — Intraday-only non-spread-normalized variant](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/bd_intra.py#L171-L371)

### G201 | Differenced Intraday Order-Flow Momentum
- priority: Low — It has a defined microstructure signal but no daily-bar mapping or execution-cost evidence.
- direction: long/short | market: US equities | timeframe: Intraday bars (intraday-only)
- indicators: First differences of ask-hit, bid-hit, and mid-hit dollar volume
- Overview: This simplified order-flow strategy uses changes in aggressive order flow rather than absolute imbalance levels, attempting to capture acceleration in buying/selling pressure.
- Stock or ETF Criteria: US equities with intraday order-flow data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: Compute (diff(askHitDollars)−diff(bidHitDollars))/(diff(askHitDollars)+diff(midHitDollars)+diff(bidHitDollars)); winsorize and industry-demean, then apply the fitted forecast coefficient.
- Exit conditions: Forecast changes at each intraday timestamp; the notebook/code does not specify a standalone exit.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Intraday-only.
- reported_metrics: none reported
- notes: No spread normalization is applied, unlike bd.py.
- sources: [noterminusgit/statarb — Differenced order-flow signal and intraday forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/bd1.py#L92-L253)

### G202 | Indicator Ternary-Signal Portfolio Engine (MACD zero-cross, Bollinger bands, CMF)
- priority: Low — Infrastructure-grade signal engine; the bootstrap preference-matrix evaluation is the reusable idea, not the signals.
- direction: long/short | market: US equities EOD | timeframe: 1d (daily-native)
- indicators: MACD(5,34) zero-cross -> event signal; Bollinger(20) band breach -> reversal signal; Chaikin Money Flow
- Overview: Book code that converts standard indicators into sparse ternary event signals (-1/0/+1 only on crossings) and simulates an equal-weight portfolio over all S&P EOD names, plus an alternative-data-driven variant. Notable less for the signals (all generic) than for the pattern: events only at signal change, portfolio-level aggregation, grid-search optimization and bootstrap preference-matrix evaluation.
- Stock or ETF Criteria: Full EOD stock universe (simulated data provided).
- Stocks to avoid and known failure conditions: Signal primitives are commodity (MACD zero-cross, BB(20) band touch) - edge must come from selection/combination, not the primitives.
- Entry conditions: MACD(5,34) sign change -> +1/-1 event; close outside Bollinger(20) band -> reversal event.
- Exit conditions: Opposite signal or holding-period rules in simulator.
- Trade management: Equal-weight across active signals; portfolio simulator with transaction costs; grid-search param optimizer; bootstrap preference matrix for parameter stability.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Bootstrap resampling exposes parameter fragility (white-noise preference matrix as control).
- Best known candlestick timeframe(s): Daily bars.
- reported_metrics: none reported
- external_refs: Chris Conlan, Algorithmic Trading with Python (2020)
- notes: MACD zero-cross -> S088 vicinity; BB band breach -> S011 vicinity.
- sources: [chrisconlan/algorithmic-trading-with-python — create_macd_signal/create_bollinger_band_signal](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/src/pypm/signals.py); [chrisconlan/algorithmic-trading-with-python — bootstrap preference matrix](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/listings/chapter_5/5_4_bootstrap_simulated_preference_matrix.py); [chrisconlan/algorithmic-trading-with-python — simulator](https://github.com/chrisconlan/algorithmic-trading-with-python/blob/ebe01087c7d9172db72bc3c9adc1eee5e882ac49/src/pypm/portfolio.py)

### G203 | MA-Slope-Biased Martingale
- priority: Low — Documented as an anti-pattern; keep the MA-slope bias idea, drop the sizing.
- direction: long/short | market: FX (MT4) | timeframe: chart TF (param) (intraday-adaptable)
- indicators: Single MA slope over BiasLookback bars for direction; martingale lot ladder
- Overview: Trade direction is set by one moving-average slope (positive = buy only); positions are added martingale-style (each additional lot = sum of open positions x IncreaseFactor). Documented here as a sizing scheme, not an edge - martingale converts many small wins into rare account-ending losses.
- Stock or ETF Criteria: FX majors.
- Stocks to avoid and known failure conditions: Martingale is the failure mode - a sustained adverse move after several additions is ruin; StartFactor 0.5 lots/1000 is aggressive.
- Entry conditions: Direction = sign(MA slope over BiasLookback); enter/add on schedule window only.
- Exit conditions: not specified in source. Martingale unwind on reversal/basket profit per EA.
- Trade management: Lot ladder: StartFactor lots per 1000, IncreaseFactor multiplication on adds; day/hour trading window gate.
- Target: Basket-level recovery.
- Risk factors and risk management: Martingale tail risk - documented for completeness, not recommended.
- Best known candlestick timeframe(s): Any.
- reported_metrics: none reported
- notes: Include for completeness; other EAs (MA Crossover, Monkey, Pedestrian, Puppy, PortfolioTrader, TraderTrainer, Multi Pair Closer) are generic or utility - logged not recorded.
- sources: [matthewkastor/Metatrader — settings](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/Biased%20Martingale/README.md); [matthewkastor/Metatrader — StartFactor/IncreaseFactor inputs](https://github.com/matthewkastor/Metatrader/blob/b10a238dbd7f0e3faa509f14a0de38e36915ae93/MQL4/Experts/Biased%20Martingale/Biased%20Martingale.mq4)

### G204 | Momentum-Following + Implementation-Shortfall Execution Study (Algo-Trading-14)
- priority: Low — Momentum+cost study; the IS-cost pairing is the lesson.
- direction: long | market: equities | timeframe: 1d (daily-native)
- indicators: Momentum-following strategies + IS (implementation shortfall) execution cost model
- Overview: Studies momentum-following strategies and their implementation-shortfall execution cost - the useful pairing of an alpha (momentum) with a real execution-cost model to evaluate net edge.
- Stock or ETF Criteria: Equities.
- Stocks to avoid and known failure conditions: Momentum signals are generic; the value is the honest cost pairing.
- Entry conditions: Momentum signal entries.
- Exit conditions: Per momentum.
- Trade management: IS execution model.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: Execution cost is the focus.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- external_refs: Implementation Shortfall execution literature
- notes: Overlap S041 momentum family.
- sources: [Weichong515/Algo-Trading-14 — momentum + IS](https://github.com/Weichong515/Algo-Trading-14/blob/471084a6863ff3edad9da430017383942a8d949e/README.md)

### G205 | Multi-Pair Martingale Bot (Binance)
- priority: Low — Anti-pattern; keep only the adverse-add (DCA) mechanic concept.
- direction: long | market: crypto (Binance) | timeframe: intraday (intraday-adaptable)
- indicators: Martingale add-ladder across multiple pairs
- Overview: Martingale averaging-down across multiple pairs simultaneously - documented as an anti-pattern reference. Loss-recovery sizing converts small wins into rare catastrophic losses.
- Stock or ETF Criteria: Binance pairs.
- Stocks to avoid and known failure conditions: Martingale = unbounded tail risk; documented for completeness only.
- Entry conditions: Initial entry + adds at adverse intervals.
- Exit conditions: Basket profit exit.
- Trade management: Martingale sizing ladder.
- Target: Basket recovery.
- Risk factors and risk management: Unbounded.
- Best known candlestick timeframe(s): Any.
- reported_metrics: none reported
- notes: Anti-pattern documentation.
- sources: [51bitquant/multi_pairs_martingle_bot — martingale bot](https://github.com/51bitquant/multi_pairs_martingle_bot/blob/d832a091ef64508a51079e97780029298bf5f89c/README.md)

### G206 | OpenAlgo Execution-Skills - VectorBT+Live Multi-Strategy Set
- priority: Low — Execution-pattern library; strategies are standard.
- direction: long/short | market: Indian markets (OpenAlgo) | timeframe: 1d + intraday (daily-native)
- indicators: VectorBT backtest strategies + OpenAlgo live execution; Indian costs; options multi-leg; portfolio risk
- Overview: Dual-mode strategy 'skills' that run identical logic in VectorBT backtests and OpenAlgo live execution, with real Indian costs/slippage and options multi-leg support - a library of executable strategy patterns.
- Stock or ETF Criteria: Indian market instruments.
- Stocks to avoid and known failure conditions: Platform-coupled.
- Entry conditions: Per skill.
- Exit conditions: Per skill.
- Trade management: Portfolio risk layer.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily + intraday.
- reported_metrics: none reported
- notes: Execution pattern resource.
- sources: [marketcalls/openalgo-execution-skills — skill files](https://github.com/marketcalls/openalgo-execution-skills/blob/1b8eceeb404f909acd2419eb329541a4de504bdc/README.md)

### G207 | Optimal Execution Algorithms Set (DP / Frank-Wolfe / Market-Impact)
- priority: Low — Execution-cost models for honest net-of-cost evaluation.
- direction: market-neutral | market: equities/crypto execution | timeframe: intra-order (intraday-only)
- indicators: Order scheduling under market-impact models
- Overview: Three repos implementing optimal-execution research: sduprey's dynamic-programming + impact model, yc3566's DP + Frank-Wolfe optimal trade execution, NikhilSehgal's Coinbase order scheduler, bhardwajshivam's HFT execution optimization. Execution-layer, not alpha - but needed to estimate cost drag on the daily strategies.
- Stock or ETF Criteria: Order-level execution.
- Stocks to avoid and known failure conditions: Not signal alpha.
- Entry conditions: N/A (execution).
- Exit conditions: N/A.
- Trade management: Optimal order slicing.
- Target: Minimize implementation shortfall.
- Risk factors and risk management: Impact model assumptions.
- Best known candlestick timeframe(s): Intra-order.
- reported_metrics: none reported
- external_refs: Almgren-Chriss / optimal execution literature
- notes: Grouped execution set.
- sources: [sduprey/optimal_transaction_execution — DP + impact](https://github.com/sduprey/optimal_transaction_execution/blob/77613ec675a1b56cc8672a9bcadccef152ed238a/README.md); [yc3566/Optimal-Trade-Execution-Algorithm-Market-Impact-Model — DP+Frank-Wolfe](https://github.com/yc3566/Optimal-Trade-Execution-Algorithm-Market-Impact-Model/blob/c7f48d79e36f8963199354d7f8b784c058b353ff/README.md); [NikhilSehgal123/coinbase-execution-algorithm — scheduler](https://github.com/NikhilSehgal123/coinbase-execution-algorithm/blob/d02ba459faf84444831fa82857a3599139e41caf/README.md); [bhardwajshivam/Optimizing-Order-Execution-Strategy-for-HFT — IS optimization](https://github.com/bhardwajshivam/Optimizing-Order-Execution-Strategy-for-HFT/blob/69b293961a70e6a72c6f7174f9ad87a527079026/README.md)

### G208 | Order-Book Insideness-Return Interaction
- priority: Low — Order-book information may be predictive but is data-intensive and execution-sensitive.
- direction: long/short | market: US equities | timeframe: Daily plus intraday bars (intraday-adaptable)
- indicators: Order-book insideness, beta-adjusted returns, industry demeaning
- Overview: The signal multiplies an order-book insideness measure by beta-adjusted returns, then fits daily and intraday forecasts. A simplified variant omits beta adjustment.
- Stock or ETF Criteria: US equities with order-book insideness, daily/intraday returns, beta and industry data.
- Stocks to avoid and known failure conditions: not specified in source Suggested: avoid low-liquidity names and regime mismatches (trend strategies in chop, mean reversion in sustained trends).
- Entry conditions: For `new1.py`, multiply beta-adjusted returns by insideness and winsorize/industry-demean; for `other2.py`, multiply raw log return by insideness. Combine lagged daily and current intraday components.
- Exit conditions: Forecast changes by timestamp; direct exit and stop rules are delegated to the simulator.
- Trade management: not specified in source Suggested: unit or volatility-scaled sizing; cap concurrent positions.
- Target: not specified in source Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Full model needs intraday data; simplified daily component may be evaluated separately.
- parameter_variants: new1.py uses beta-adjusted returns; other2.py omits beta adjustment.
- reported_metrics: none reported
- notes: The source modules emit alpha forecasts rather than complete trade management.
- sources: [noterminusgit/statarb — Beta-adjusted return/insideness signal, daily/intraday fits and forecast](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/new1.py#L56-L162); [noterminusgit/statarb — Simplified raw-return/insideness interaction](https://github.com/noterminusgit/statarb/blob/b5a981ab2658b5eb5cd0a3f610bc9f4db7142655/other2.py#L42-L161)

### G209 | ai-trader - Backtrader 20+ Strategy Collection
- priority: Low — Baseline strategy collection overlapping catalog families.
- direction: long/short | market: multi-market | timeframe: 1d (daily-native)
- indicators: 20+ backtrader strategies (MA crosses, RSI, MACD, Bollinger, mean reversion, momentum, etc.)
- Overview: Backtrader-powered collection of 20+ ready strategies with multi-market support and CLI - the strategies are the usual canonical set (SMA/EMA crosses, RSI, MACD, Bollinger MR, breakout, momentum). Useful as a second clean reference for baseline strategies on daily bars.
- Stock or ETF Criteria: Multi-market via backtrader feeds.
- Stocks to avoid and known failure conditions: Canonical signals; parameter defaults untuned.
- Entry conditions: Per strategy file.
- Exit conditions: Per strategy.
- Trade management: backtrader sizing.
- Target: not specified in source. Suggested: exit at the signal's natural objective or a multiple of ATR.
- Risk factors and risk management: not specified in source. Suggested: per-trade stop and a portfolio-level drawdown guard.
- Best known candlestick timeframe(s): Daily.
- reported_metrics: none reported
- notes: Overlap S001/S011/S088/S089/S036 vicinity.
- sources: [whchien/ai-trader — classic strategy files (+3 more verified paths)](https://github.com/whchien/ai-trader/blob/7681fe331fb638beab1ae20d3552c802bbaa9174/ai_trader/backtesting/strategies/classic/adaptive_rsi.py); [whchien/ai-trader — ai_trader/backtesting/strategies/classic/bbands.py](https://github.com/whchien/ai-trader/blob/7681fe331fb638beab1ae20d3552c802bbaa9174/ai_trader/backtesting/strategies/classic/bbands.py); [whchien/ai-trader — ai_trader/backtesting/strategies/](https://github.com/whchien/ai-trader/blob/7681fe331fb638beab1ae20d3552c802bbaa9174/ai_trader/backtesting/strategies/)
