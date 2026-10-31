# GitHub Trading Algorithm Repository Inventory

Compiled October 28, 2026, from the GitHub repository search API: 309 candidate repositories in 12 categories, plus 29 supporting tools and unresolved candidates.

## Scope & Method

This is a broad categorized inventory, not a literally complete list: a single query like "trading algorithm" alone matches 17,000+ repositories, and GitHub caps any search at 1,000 results. The inventory was built from 24 family-specific GitHub searches (trading algorithm, algorithmic trading, pairs trading, statistical arbitrage, mean reversion, market making, crypto trading bot, ML/RL trading, options strategies, execution algorithms, grid/DCA, trend following, arbitrage, and more), retrieving the top 25 star-ranked results from 14 of them and the top 10 from the other 10, plus one direct lookup for Microsoft's Qlib. Results were deduplicated and categorized by hand.

It covers equities, crypto, FX, futures, options and prediction markets, across Python, C#, C++, Rust, Go, TypeScript, Java, R and others. Entries are included based on their own repository descriptions as returned by the GitHub API; selected READMEs of the most-starred entries were checked for descriptions of trading functionality, but individual strategy implementations were not code-audited. Treat the main entries as candidates. Star counts are as of the collection date. Archived means the repository is read-only; it says nothing about successor projects. `ccxt/ccxt` is excluded because it is an exchange-API library rather than a trading algorithm; curated lists are linked at the end as further discovery resources.

## Frameworks & platforms (31)

Engines and platforms where you write and run your own trading algorithms (backtesting and/or live execution). These implement the infrastructure; the strategies live in user code, though most ship example strategies.

- **[microsoft/qlib](https://github.com/microsoft/qlib)** — 49,785 stars — Python: AI-oriented quantitative investment platform covering the full ML research-to-production pipeline (supervised learning, market dynamics, RL).
- **[nautechsystems/nautilus_trader](https://github.com/nautechsystems/nautilus_trader)** — 31,014 stars — Rust: Production-grade Rust-native trading engine with deterministic event-driven architecture
- **[mementum/backtrader](https://github.com/mementum/backtrader)** — 23,607 stars — Python: Python Backtesting library for trading strategies
- **[QuantConnect/Lean](https://github.com/QuantConnect/Lean)** — 22,285 stars — C#: Lean Algorithmic Trading Engine by QuantConnect (Python, C#)
- **[quantopian/zipline](https://github.com/quantopian/zipline)** — 20,173 stars — Python: Zipline, a Pythonic Algorithmic Trading Library
- **[UFund-Me/Qbot](https://github.com/UFund-Me/Qbot)** — 18,748 stars — Jupyter Notebook: AI-powered quantitative investment research platform with fully local deployment.
- **[goldmansachs/gs-quant](https://github.com/goldmansachs/gs-quant)** — 13,794 stars — Python: Python toolkit for quantitative finance
- **[StockSharp/StockSharp](https://github.com/StockSharp/StockSharp)** — 11,018 stars — C#: Algorithmic trading and quantitative trading open source platform to develop trading robots (stock markets, forex, crypto, bitcoins, and options).
- **[polakowo/vectorbt](https://github.com/polakowo/vectorbt)** — 9,576 stars — Python: The research engine that gives you an unfair advantage. Run thousands of trading ideas before others finish one.
- **[kernc/backtesting.py](https://github.com/kernc/backtesting.py)** — 9,155 stars — Python: Backtest trading strategies in Python.
- **[ricequant/rqalpha](https://github.com/ricequant/rqalpha)** — 6,843 stars — Python: A extendable, replaceable Python algorithmic backtest && trading framework supporting multiple securities
- **[charliedream1/ai_quant_trade](https://github.com/charliedream1/ai_quant_trade)** — 6,665 stars — Jupyter Notebook: Stock AI Trader: one-stop platform for learning, simulation and live trading — strategies, LLMs, factor mining, ML/DL/RL, graph nets, HFT and deployment.
- **[gbeced/pyalgotrade](https://github.com/gbeced/pyalgotrade)** — 4,677 stars — Python — archived (read-only): Python Algorithmic Trading Library
- **[shashankvemuri/Finance](https://github.com/shashankvemuri/Finance)** — 4,357 stars — Python: Python toolkit for quantitative finance: stock analysis, technical indicators, strategy backtesting, portfolio optimization, and financial modeling.
- **[zvtvz/zvt](https://github.com/zvtvz/zvt)** — 4,318 stars — Python: modular quant framework.
- **[cuemacro/finmarketpy](https://github.com/cuemacro/finmarketpy)** — 3,822 stars — Python: Python library for backtesting trading strategies & analyzing financial markets (formerly pythalesians)
- **[edtechre/pybroker](https://github.com/edtechre/pybroker)** — 3,665 stars — Python: Algorithmic Trading in Python with Machine Learning
- **[blankly-finance/blankly](https://github.com/blankly-finance/blankly)** — 2,483 stars — Python: Easily build, backtest and deploy your algo in just a few lines of code. Trade stocks, cryptos, and forex across exchanges w/ one package.
- **[ranaroussi/qtpylib](https://github.com/ranaroussi/qtpylib)** — 2,267 stars — Python — archived (read-only): QTPyLib, Pythonic Algorithmic Trading
- **[coding-kitties/investing-algorithm-framework](https://github.com/coding-kitties/investing-algorithm-framework)** — 2,204 stars — Python: Framework for quantitative trading. Complete framework for development, backtesting, and deploying automated trading algorithms and trading bots.
- **[Lumiwealth/lumibot](https://github.com/Lumiwealth/lumibot)** — 2,182 stars — Python: Python trading framework with AI agents that actually place the trade. Backtest, paper and live trade on 12 brokers: stocks, options, futures, forex, crypto and prediction markets.
- **[austin-starks/NextTrade](https://github.com/austin-starks/NextTrade)** — 1,776 stars — TypeScript: (Predecessor to NexusTrade) A no-code algorithmic trading system
- **[whchien/ai-trader](https://github.com/whchien/ai-trader)** — 1,121 stars — Python: Backtrader-powered backtesting framework for algorithmic trading, featuring 20+ strategies, multi-market support, CLI tools, and an integrated MCP server for professional traders.
- **[Kismuz/btgym](https://github.com/Kismuz/btgym)** — 1,036 stars — Python: Scalable, event-driven, deep-learning-friendly backtesting library
- **[EA31337/EA31337-classes](https://github.com/EA31337/EA31337-classes)** — 275 stars — MQL5: EA31337 framework (MQL library for writing trading Expert Advisors, indicators and scripts)
- **[kay-ou/ptradeAPI](https://github.com/kay-ou/ptradeAPI)** — 205 stars — Python: PTrade is a full‑featured quantitative trading platform for developing, backtesting and executing strategies across stocks, convertible bonds, futures, ETFs, and options. This project provides API wrappers and documen...
- **[dsinyakov/quant](https://github.com/dsinyakov/quant)** — 183 stars — Java: Codera Quant is a Java framework for algorithmic trading strategies development, execution and backtesting via Interactive Brokers TWS API or other brokers API
- **[Markfans/cryptoquant-ai](https://github.com/Markfans/cryptoquant-ai)** — 182 stars — TypeScript: CryptoQuant AI is an advanced, open-source quantitative trading platform designed to bridge the gap between algorithmic market execution and artificial intelligence. Built entirely on a modern Node.js, Vite, and TypeS...
- **[tfukaza/harvest](https://github.com/tfukaza/harvest)** — 152 stars — Python — archived (read-only): Simple and intuitive Python framework for algorithmic trading. Easily create bots to live and paper trade stocks, crypto, and options!
- **[automatedalgo/apex](https://github.com/automatedalgo/apex)** — 102 stars — C++: Algorithmic trading strategies research and execution platform
- **[Thomvanoorschot/zigma](https://github.com/Thomvanoorschot/zigma)** — 101 stars — Zig: Zigma is an algorithmic trading framework built with the Zig programming language, leveraging an actor-based concurrency model. It aims to provide an efficient, low-latency system for algorithmic trading through compo...

## Crypto trading bots (20)

End-to-end crypto bot platforms and ready-made strategy collections implementing concrete trading algorithms.

- **[freqtrade/freqtrade](https://github.com/freqtrade/freqtrade)** — 56,104 stars — Python: Free, open source crypto trading bot
- **[askmike/gekko](https://github.com/askmike/gekko)** — 10,180 stars — JavaScript — archived (read-only): A bitcoin trading bot written in node - https://gekko.wizb.it/
- **[jesse-ai/jesse](https://github.com/jesse-ai/jesse)** — 8,729 stars — Python: An advanced crypto trading bot written in Python
- **[Drakkar-Software/OctoBot](https://github.com/Drakkar-Software/OctoBot)** — 6,873 stars — Python: Free open source crypto trading bot to automate AI, Grid, DCA and TradingView strategies on Binance, Hyperliquid and 15+ exchanges, with a simple interface.
- **[Superalgos/Superalgos](https://github.com/Superalgos/Superalgos)** — 5,739 stars — JavaScript: Free, open-source crypto trading bot, automated bitcoin / cryptocurrency trading software, algorithmic trading bots. Visually design your crypto trading bot, leveraging an integrated charting system, data-mining, back...
- **[CryptoSignal/Crypto-Signal](https://github.com/CryptoSignal/Crypto-Signal)** — 5,653 stars — Python: Github.com/CryptoSignal - Trading & Technical Analysis Bot - 4,100+ stars, 1,100+ forks
- **[freqtrade/freqtrade-strategies](https://github.com/freqtrade/freqtrade-strategies)** — 5,603 stars — Python: Free trading strategies for Freqtrade bot
- **[Haehnchen/crypto-trading-bot](https://github.com/Haehnchen/crypto-trading-bot)** — 3,543 stars — TypeScript: Cryptocurrency trading bot in javascript for Bitfinex, Bitmex, Binance, Bybit ... (public edition)
- **[iterativv/NostalgiaForInfinity](https://github.com/iterativv/NostalgiaForInfinity)** — 3,500 stars — Python: Trading strategy for the Freqtrade crypto bot
- **[scrtlabs/catalyst](https://github.com/scrtlabs/catalyst)** — 2,559 stars — Python — archived (read-only): An Algorithmic Trading Library for Crypto-Assets in Python
- **[warp-id/solana-trading-bot](https://github.com/warp-id/solana-trading-bot)** — 2,338 stars — TypeScript: Solana Trading Bot - Beta
- **[whittlem/pycryptobot](https://github.com/whittlem/pycryptobot)** — 2,059 stars — Python: Python Crypto Bot (PyCryptoBot)
- **[ivopetiz/algotrading](https://github.com/ivopetiz/algotrading)** — 1,703 stars — Python: Algorithmic trading framework for cryptocurrencies.
- **[c9s/bbgo](https://github.com/c9s/bbgo)** — 1,674 stars — Go: The modern cryptocurrency trading bot framework written in Go.
- **[CyberPunkMetalHead/Binance-News-Sentiment-Bot](https://github.com/CyberPunkMetalHead/Binance-News-Sentiment-Bot)** — 1,653 stars — Python: This is a fully functioning Binance trading bot that takes into account the news sentiment for the top 100 crypto feeds. If you like this project consider donating though the Brave browser to allow me to continuously...
- **[rodrigo-brito/ninjabot](https://github.com/rodrigo-brito/ninjabot)** — 1,635 stars — Go: A fast trading bot platform for cryptocurrency in Go (Binance)
- **[xFFFFF/Gekko-Strategies](https://github.com/xFFFFF/Gekko-Strategies)** — 1,441 stars — JavaScript: Strategies to Gekko trading bot with backtests results and some useful tools.
- **[Superior-Trade/trading-terminal](https://github.com/Superior-Trade/trading-terminal)** — 1,004 stars — TypeScript: Self-hosted AI trading terminal for Hyperliquid: chart read in, validated Freqtrade/Nautilus-Trader strategy out, deployed live.
- **[51bitquant/multi_pairs_martingle_bot](https://github.com/51bitquant/multi_pairs_martingle_bot)** — 126 stars — Python: A muti pairs martingle trading bot for Binance exchange.
- **[Stenvro/ApexAlgo](https://github.com/Stenvro/ApexAlgo)** — 51 stars — Python: Drag-and-drop algorithmic trading bot builder for crypto. Features live charting, built-in risk management (kill-switches), and asynchronous execution via FastAPI and CCXT.

## Trend & momentum (12)

Trend-following, momentum, breakout and CTA-style strategy implementations.

- **[popbo/alphas](https://github.com/popbo/alphas)** — 599 stars — Jupyter Notebook: alpha101, alpha191, alphalens, backtrader, 量化研究
- **[lamres/renko_trend_following_strategy_catalyst](https://github.com/lamres/renko_trend_following_strategy_catalyst)** — 126 stars — Jupyter Notebook: Example of adaptive trend following strategy based on Renko
- **[9nix6/Median-and-Turbo-Renko-indicator-bundle](https://github.com/9nix6/Median-and-Turbo-Renko-indicator-bundle)** — 100 stars — MQL5: Open-source MetaTrader 5 SDK for Expert Advisors and indicators that run directly on Median & Turbo Renko charts. Works with both the MQL5 Market edition and Ultimate Renko from az-invest.eu — client library, 64 porte...
- **[kieranjwood/x-trend](https://github.com/kieranjwood/x-trend)** — 96 stars: X-Trend: Few-Shot Learning Patterns in Financial Time-Series for Trend-Following Strategies
- **[tschm/cs](https://github.com/tschm/cs)** — 47 stars — Python: The 10-line CTA: a trend-following strategy in ten lines of code, and four refinements measured in Sharpe ratio, kurtosis and trading costs
- **[quantrocket-codeload/trend-day](https://github.com/quantrocket-codeload/trend-day)** — 28 stars — Jupyter Notebook: Intraday momentum strategy that trades leveraged ETFs late in the session after a significant intraday gain/loss and holds to the close (from Ernie Chan's book Algorithmic Trading).
- **[tschm/TinyCTA](https://github.com/tschm/TinyCTA)** — 18 stars — Python: Building blocks for trend-following CTA strategies: scale-free oscillators, volatility-adjusted returns, matrix shrinkage and a position engine
- **[timchepeleff/turtles](https://github.com/timchepeleff/turtles)** — 18 stars — JavaScript: Turtle trend following strategy implemented for various algorithmic trading frameworks
- **[rahulmr/Pinescript-SuperTrend-Trading-Strategy](https://github.com/rahulmr/Pinescript-SuperTrend-Trading-Strategy)** — 16 stars: A trend-following strategy triggering longs & shorts on Tradingview charts.
- **[Mortiniera/algorithmic-trading-technical-indicators](https://github.com/Mortiniera/algorithmic-trading-technical-indicators)** — 14 stars — HTML: Python implementation of simple algorithmic trading strategies using Momentum and Trend following technical indicators used by traders and investors in financial markets to analyze past market data and identify potent...
- **[0a1b/momentum_NautilusTrader](https://github.com/0a1b/momentum_NautilusTrader)** — 12 stars — Jupyter Notebook: Classic Momentum Strategy with Nautilus Trader (AI Slob)
- **[blacksouane/QARMII_project](https://github.com/blacksouane/QARMII_project)** — 6 stars — MATLAB: Trend-Following strategies (CTA style) to reduce downside risk and high correlation exposure.

## Mean reversion & stat arb (50)

Pairs trading, cointegration, mean-reversion and statistical-arbitrage implementations.

- **[jamesmawm/High-Frequency-Trading-Model-with-IB](https://github.com/jamesmawm/High-Frequency-Trading-Model-with-IB)** — 2,944 stars — Python: A high-frequency trading model using Interactive Brokers API with pairs and mean-reversion in Python
- **[KidQuant/Pairs-Trading-With-Python](https://github.com/KidQuant/Pairs-Trading-With-Python)** — 853 stars — Jupyter Notebook: This project involves using a combination of statistics along with financial thoery to demonstrate a popular trading strategy used in equity markets: Pairs Trading.
- **[rolling-panda-san/notebooks](https://github.com/rolling-panda-san/notebooks)** — 777 stars — Jupyter Notebook: Analysis on systematic trading strategies (e.g., trend-following, carry and mean-reversion). The result is regularly updated.
- **[lukstei/trading-backtest](https://github.com/lukstei/trading-backtest)** — 352 stars — Java: A stock backtesting engine written in Java. And a pairs trading (cointegration) strategy implementation using a bayesian kalman filter model
- **[financialnoob/pairs_trading](https://github.com/financialnoob/pairs_trading)** — 349 stars — Jupyter Notebook: experiments with pair trading
- **[shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading](https://github.com/shenyichen105/Deep-Reinforcement-Learning-in-Stock-Trading)** — 323 stars — Python: Using deep actor-critic model to learn best strategies in pair trading
- **[gregzanotti/dlsa-public](https://github.com/gregzanotti/dlsa-public)** — 279 stars — Python: Deep Learning Statistical Arbitrage
- **[wywongbd/pairstrade-fyp-2019](https://github.com/wywongbd/pairstrade-fyp-2019)** — 273 stars — Python: We tested 3 approaches for Pair Trading: distance, cointegration and reinforcement learning approach.
- **[marketneutral/pairs-trading-with-ML](https://github.com/marketneutral/pairs-trading-with-ML)** — 219 stars — Jupyter Notebook: 
- **[SpencerPao/Quantitative_Strategies](https://github.com/SpencerPao/Quantitative_Strategies)** — 217 stars — Jupyter Notebook: This includes a notebook on how to implement Quantitative Strategies, specifically the Pairs Trading Algorithm.
- **[tibkiss/huba-v1](https://github.com/tibkiss/huba-v1)** — 200 stars — Python: Pairs Trading using Statistical Arbitrage
- **[simaomsarmento/PairsTrading](https://github.com/simaomsarmento/PairsTrading)** — 180 stars — Jupyter Notebook: Repository containing the code for a pairs trading investment strategy (Master Thesis in Electrical and Computer Engineering - Técnico Lisboa).
- **[hudson-and-thames/arbitrage_research](https://github.com/hudson-and-thames/arbitrage_research)** — 161 stars — Jupyter Notebook: Jupyter Notebook examples on how to use the ArbitrageLab - pairs trading - python library.
- **[The-FinAI/trials](https://github.com/The-FinAI/trials)** — 136 stars — Python: trials: Select and Trade — unified pair trading with hierarchical reinforcement learning (arXiv:2301.10724)
- **[wangy8989/Pairs-Trading-with-Machine-Learning](https://github.com/wangy8989/Pairs-Trading-with-Machine-Learning)** — 129 stars — Python: Pairs Trading with Machine Learning on Distributed Python Platform
- **[Jackal08/QuantInsti-Final-Project-Statistical-Arbitrage](https://github.com/Jackal08/QuantInsti-Final-Project-Statistical-Arbitrage)** — 117 stars — R: QuantInsti EPAT: Final Project on Statistical Arbitrage
- **[sapphire921/HFT-Pairs-Trading](https://github.com/sapphire921/HFT-Pairs-Trading)** — 107 stars — Jupyter Notebook: High Frequency Pairs Trading Based on Statistical Arbitrage (Python)
- **[jerryxyx/TreasuryFutureTrading](https://github.com/jerryxyx/TreasuryFutureTrading)** — 88 stars — Jupyter Notebook: A statistical arbitrage strategy on treasury futures using mean-reversion property and meanwhile insensitive to the yield change
- **[orangepineapple/mean_reversion_strategies](https://github.com/orangepineapple/mean_reversion_strategies)** — 83 stars — Jupyter Notebook: Here I go through the processing of prototyping a mean reversion trading strategy using statistical concepts, then test it in backtrader.
- **[YINDAIYING/Deep-Robust-Statistical-Arbitrage](https://github.com/YINDAIYING/Deep-Robust-Statistical-Arbitrage)** — 76 stars — Jupyter Notebook: This repository stores the implementation of the paper "DETECTING DATA-DRIVEN ROBUST STATISTICAL ARBITRAGE STRATEGIES WITH DEEP NEURAL NETWORKS"
- **[sap215/StatArbPairsTrading](https://github.com/sap215/StatArbPairsTrading)** — 71 stars — Jupyter Notebook: This repository includes an introduction to statistical arbitrage pairs trading. Specifically, I discuss some of the research methods required in order to find a successful pair as well as the code implementation for...
- **[bideeen/Building-A-Trading-Strategy-With-Python](https://github.com/bideeen/Building-A-Trading-Strategy-With-Python)** — 65 stars — Jupyter Notebook: trading strategy is a fixed plan to go long or short in markets, there are two common trading strategies: the momentum strategy and the reversion strategy. Firstly, the momentum strategy is also called divergence or t...
- **[shimonanarang/pair-trading](https://github.com/shimonanarang/pair-trading)** — 61 stars — Python: Statistical arbitrage of cointegrating currencies with pair trading where the signal for the next day is predicted using LSTM
- **[harpone/PyArb](https://github.com/harpone/PyArb)** — 57 stars — HTML: Statistical arbitrage simulation, modeling and backtesting with Python.
- **[BessieChen/R-in-Finance](https://github.com/BessieChen/R-in-Finance)** — 53 stars — R: This course, taught by Prof.Jerzy in NYU, applies the R programming language to momentum trading, statistical arbitrage (pairs trading), and other active portfolio management strategies. The course implements volatili...
- **[linhnguyen215538/Volatility-Study](https://github.com/linhnguyen215538/Volatility-Study)** — 52 stars — Jupyter Notebook: • Conducted a volatility study to develop pairs trading strategy by writing web crawlers that automated extracting 30 equity and ETF spot and options prices data from CBOE and Yahoo Finance • Utilized NumPy, Pandas, a...
- **[theanh97/Statistical-Arbitrage-Bayesian-Optimized-Kappa-Half-life-Pairs-Trading-Engine](https://github.com/theanh97/Statistical-Arbitrage-Bayesian-Optimized-Kappa-Half-life-Pairs-Trading-Engine)** — 47 stars — Jupyter Notebook: This project implements an advanced pairs trading strategy using statistical arbitrage techniques. It leverages Bayesian optimization to fine-tune Kappa and Half-life parameters, enhancing the mean-reversion trading a...
- **[edgetrader/mean-reversion-strategy](https://github.com/edgetrader/mean-reversion-strategy)** — 31 stars — Jupyter Notebook: Mean Reversion Trading Strategy
- **[DuarteAbracos/statistical-arbitrage-kalman](https://github.com/DuarteAbracos/statistical-arbitrage-kalman)** — 28 stars — Jupyter Notebook: Intraday pairs trading engine using Kalman Filters for dynamic beta estimation and Avellaneda-Stoikov optimal execution.
- **[BananaHamm/Equity_StatArb](https://github.com/BananaHamm/Equity_StatArb)** — 28 stars — Python: Implementation for "Statistical arbitrage in the US equities market" by Marco Avellaneda and Jeong-hyun Lee
- **[notaconduit/Statistical-Arbitrage-in-Cryptocurrencies](https://github.com/notaconduit/Statistical-Arbitrage-in-Cryptocurrencies)** — 27 stars — Python: The goal of this project is to develop a statistical arbitrage strategy for cryptocurrencies using Python
- **[justinlent/PairTradeR](https://github.com/justinlent/PairTradeR)** — 27 stars — R: Web GUI for backtesting pair trading statistical arbitrage portfolio strategies
- **[titouannwtt/freqtrade-ultimate](https://github.com/titouannwtt/freqtrade-ultimate)** — 25 stars — Python: Freqtrade fork for algorithmic trading on Hyperliquid perpetual futures: multi-bot OHLCV/pairlist caching, overfitting-resistant hyperopt (PlateauSampler, walk-forward, CPCV), custom losses, liquidation detection and...
- **[noterminusgit/statarb](https://github.com/noterminusgit/statarb)** — 23 stars — Python: Production-grade statistical arbitrage trading system with 20+ alpha strategies, portfolio optimization, and multiple backtesting engines. Features PCA decomposition, Barra risk models, and realistic transaction cost...
- **[yungalyx/NoisyBoyAlgotrader](https://github.com/yungalyx/NoisyBoyAlgotrader)** — 16 stars — Python: algorithmic trading strategy built backtested using backtrader and python, optimizing risk-adjusted returns with a bollinger mean-reversion strategy
- **[quantrocket-codeload/calspread](https://github.com/quantrocket-codeload/calspread)** — 16 stars — Jupyter Notebook: Intraday trading strategy for futures calendar spreads. Uses crude oil futures and 1-minute bid/ask bars from Interactive Brokers with a Bollinger Band mean reversion strategy. Runs in Moonshot. Demonstrates using exc...
- **[eshan-kaul/PairsTrading-Crypto](https://github.com/eshan-kaul/PairsTrading-Crypto)** — 12 stars — Jupyter Notebook: An emerging asset class, the recent surge of popularity in crypto markets has made cryptocurrencies an essential part of investment portfolios for retail and institutional investors. As prices for cryptocurrencies con...
- **[stephenkyang/mean-reversion-pairs-trading](https://github.com/stephenkyang/mean-reversion-pairs-trading)** — 11 stars — Python: manipulating cointegrated pairs to achieve a market-neutral strategy that outperforms indices
- **[MXGao-A/TradeRules](https://github.com/MXGao-A/TradeRules)** — 11 stars — Python: A rule-based backtesting framework that combines mean reversion and trend following strategies for systematic trading analysis.
- **[XBT3K/MeanReversionAlgo](https://github.com/XBT3K/MeanReversionAlgo)** — 10 stars — Python: Python algorithm for trading the EUR/USD forex pair using a mean reversion strategy. The algorithm retrieves price data from OANDA's API, calculates the z-score of the closing prices, and executes a trade if the z-sco...
- **[XanderRobbins/Universal-Pairs-Trading-System](https://github.com/XanderRobbins/Universal-Pairs-Trading-System)** — 10 stars — Python: A professional-grade quantitative trading system that implements statistical arbitrage through mean-reversion strategies on cointegrated asset pairs.
- **[guldo111/binance-mean-reversion-trading-bot](https://github.com/guldo111/binance-mean-reversion-trading-bot)** — 10 stars — Python: Automated trading bot powered by CCXT library, implementing a mean reversion strategy for cryptocurrencies on Binance.
- **[BearsOnMars/Algorithmic_Trading](https://github.com/BearsOnMars/Algorithmic_Trading)** — 10 stars — Jupyter Notebook: This repository contains python code to create, backtest and automate intraday-trading algorithms in financial markets using Machine Learning (Regression, Classification) and Statistical (Mean-Reversion, Moving Averag...
- **[Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot](https://github.com/Xtley001/Mean-Reversion-Gap-Strategy-Trading-Bot)** — 9 stars — Python: A sophisticated algorithmic trading system designed for prop firm challenges, implementing mean reversion strategies with advanced risk management and multi-timeframe analysis on MetaTrader 5.
- **[FarzamHejaziK/claude-investment-assistant](https://github.com/FarzamHejaziK/claude-investment-assistant)** — 9 stars: A Claude Code workspace template that automates your investing every morning. Rules over emotion. No fear, no FOMO. Ships with 3 default strategies — dip-buying, AI value-chain DCA, and mean-reversion active trading —...
- **[imad08/Mean-Reversion-Trading-Strategy](https://github.com/imad08/Mean-Reversion-Trading-Strategy)** — 8 stars — Jupyter Notebook: This repository is for codes of project MRTS under Finance And Analytics club IITK
- **[honoreaa/Futures-MeanReversion](https://github.com/honoreaa/Futures-MeanReversion)** — 7 stars — Jupyter Notebook: Pairs Trading Strategy: ES=F and NQ=F Mean Reversion
- **[garvit-kudesia91/Statistical-Arbitrage](https://github.com/garvit-kudesia91/Statistical-Arbitrage)** — 7 stars — Jupyter Notebook: Pairs Trading using Augmented Dickey Fuller Test. This widget can be used to find co-integrated pairs of ETFs from all the currently listed Vanguard ETFs (74 ETFs as of 7th October 2017) on a time frame of 1 year. Usi...
- **[iamshakibulislam/gbp-usd-forex-trading-mean-reversion-bot](https://github.com/iamshakibulislam/gbp-usd-forex-trading-mean-reversion-bot)** — 6 stars — MQL4: This is a forex trading bot uses mean reversal strategy to trade - This bot generated over 76% per year on average since 2011 . This bot is for only meta trader 4
- **[XBT3K/RSI-MA-EURUSD-Algo](https://github.com/XBT3K/RSI-MA-EURUSD-Algo)** — 6 stars — Python: This is a modified mean reversion trading strategy that generates buy and sell signals based on the relative strength index (RSI) and moving average (MA) of the EUR/USD currency pair on a 1-minute timeframe.

## Market making & HFT (24)

Market-making quoting strategies and high-frequency trading engines, including limit-order-book backtesting tools.

- **[nkaz001/hftbacktest](https://github.com/nkaz001/hftbacktest)** — 5,020 stars — Rust: Free, open source, a high frequency trading and market making backtesting and trading bot, which accounts for limit orders, queue positions, and latencies, utilizing full tick data for trades and order books(Level-2 a...
- **[ctubio/Krypto-trading-bot](https://github.com/ctubio/Krypto-trading-bot)** — 3,727 stars — C++: Self-hosted crypto trading bot (automated high frequency market making) written in C++
- **[warproxxx/poly-maker](https://github.com/warproxxx/poly-maker)** — 1,516 stars — Python: An automated market making bot for the Polymarket Prediction Market that provides liquidity by maintaining orders on both sides of the order book with customizable parameters
- **[purefinance/mmb](https://github.com/purefinance/mmb)** — 616 stars — Rust: Trading bot implemented in Rust, with market making and strategy automation for any exchange or blockchain.
- **[ondra-novak/mmbot](https://github.com/ondra-novak/mmbot)** — 254 stars — C++: Market Making trading bot for cryptomarkets
- **[zbarge/stocklook](https://github.com/zbarge/stocklook)** — 171 stars — Python: crypto currency library for trading & market making bots, account management, and data analysis
- **[Polytutor-AI-Labs/predictfun-market-maker-trading-bot](https://github.com/Polytutor-AI-Labs/predictfun-market-maker-trading-bot)** — 149 stars — Python: Passive market-making bot for Predict.fun: quotes from live order books, replaces stale orders and cancels on unsafe price/depth; Windows desktop app and web console for Ubuntu servers.
- **[kachence/polymm](https://github.com/kachence/polymm)** — 113 stars — Python: A Polymarket sports market-making / arbitrage bot in Python - de-vig sportsbook odds, quote, hedge. The retired code behind a public $5k wallet.
- **[algotrading-lab/coinbase-market-making-bot](https://github.com/algotrading-lab/coinbase-market-making-bot)** — 112 stars — TypeScript: coinbase market making bot to provide liquidity by quoting bid and ask coinbase market making bot coinbase market making bot
- **[K0414/algo-trader-tool-suite](https://github.com/K0414/algo-trader-tool-suite)** — 91 stars — C++: Proprietary trading solution for high-frequency trading (HFT) and statistical arbitrage algorithms
- **[rlindland/options-market-making](https://github.com/rlindland/options-market-making)** — 89 stars — Python: A bot coded for an algorithmic trading competition using market making, statistical arbitrage, and delta and vega hedging
- **[grzresearch/Hyperliquid-Outcome-Market-Making-Bot-](https://github.com/grzresearch/Hyperliquid-Outcome-Market-Making-Bot-)** — 50 stars — Rust: 
- **[manifoldmarkets/market-maker](https://github.com/manifoldmarkets/market-maker)** — 45 stars — TypeScript: A market-making bot for Manifold's prediction markets
- **[Drakkar-Software/OctoBot-Market-Making](https://github.com/Drakkar-Software/OctoBot-Market-Making)** — 42 stars — Python: Crypto market making strategy automation software for MEXC, Bitmart, Binance and 15+ exchanges based on OctoBot and available as an open source trading bot.
- **[vincent212/kaspar-hft](https://github.com/vincent212/kaspar-hft)** — 41 stars — C++: C++20 production HFT system and order book simulator. Turn-key execution algorithm (percentage-of-volume) that follows natural order flow. Queue-position-aware order book simulator, MDP3 decoder, iLink 3, PCAP replay,...
- **[haohaoi34/nexus-trade-bot](https://github.com/haohaoi34/nexus-trade-bot)** — 38 stars — Go: Millisecond-level high-frequency crypto market-making and grid trading system with web console, risk controls and self-hosted deployment.
- **[terrytrl100/polymarket-automated-mm](https://github.com/terrytrl100/polymarket-automated-mm)** — 36 stars — Python: Automated market making bot for Polymarket prediction markets
- **[bitbankinc/sample-market-making-bot](https://github.com/bitbankinc/sample-market-making-bot)** — 35 stars — JavaScript: Sample market making bot at bitbank.cc
- **[mxaddict/mmaker](https://github.com/mxaddict/mmaker)** — 32 stars — JavaScript — archived (read-only): MMaker is a Market Making trading bot
- **[evermarkets/market-maker-bot](https://github.com/evermarkets/market-maker-bot)** — 27 stars — Python: Market making strategy example
- **[quedexnet/java-market-maker](https://github.com/quedexnet/java-market-maker)** — 26 stars — Java: Simple Market Making bot for Quedex Bitcoin Derivatives Exchange.
- **[HydroProtocol/amm-bots](https://github.com/HydroProtocol/amm-bots)** — 23 stars — Go: Automated market making bots to provide liquidity for hydro relayers
- **[andydardgallard/FarukonAlgoTradingPlatform](https://github.com/andydardgallard/FarukonAlgoTradingPlatform)** — 9 stars — Rust: Farukon is a high-performance, Rust-based algorithmic trading platform designed for ultra-low-latency backtesting, multi-strategy optimization, and real-time execution. Built with zero-copy FlatBuffers, SIMD-accelerat...
- **[kukuhtw/dma_bot_rust](https://github.com/kukuhtw/dma_bot_rust)** — 8 stars — Rust: fast, async trading engine in Rust for crypto. It streams multi-symbol market data (mock or Binance), runs pluggable strategies (mean-reversion, MA crossover, volatility breakout), applies risk limits, routes orders a...

## Arbitrage (28)

Cross-exchange, triangular, DEX and flash-loan arbitrage implementations.

- **[hummingbot/hummingbot](https://github.com/hummingbot/hummingbot)** — 20,774 stars — Python: Open source software that helps you create and deploy high-frequency crypto trading bots
- **[alsk1992/CloddsBot](https://github.com/alsk1992/CloddsBot)** — 3,016 stars — TypeScript: Open Source AI trading agent that operates autonomously across 1000+ markets - Polymarket, Kalshi, Binance, Hyperliquid, Solana DEXs, 5 EVM chains. Scans for edge, executes instantly, manages risk while you sleep. Age...
- **[Roibal/Cryptocurrency-Trading-Bots-Python-Beginner-Advance](https://github.com/Roibal/Cryptocurrency-Trading-Bots-Python-Beginner-Advance)** — 1,443 stars — Python: Crypto Trading Bots in Python - Triangular Arbitrage, Beginner & Advanced Cryptocurrency Trading Bots Written in Python
- **[kelvinau/crypto-arbitrage](https://github.com/kelvinau/crypto-arbitrage)** — 851 stars — Python: Automatic Cryptocurrency Trading Bot using Triangular or Exchange Arbitrages
- **[hzjken/crypto-arbitrage-framework](https://github.com/hzjken/crypto-arbitrage-framework)** — 706 stars — Python: A cryptocurrency arbitrage framework implemented with ccxt and cplex. It can be used to monitor multiple exchanges, find a multi-lateral arbitrage path which maximizes rate of return, calculate the optimal trading amo...
- **[featherenvy/botvana](https://github.com/featherenvy/botvana)** — 250 stars — Rust: Botvana is high-performance and event-driven trading system built using Rust (in early development).
- **[andrei-zgirvaci/Arbitrage-Bot](https://github.com/andrei-zgirvaci/Arbitrage-Bot)** — 248 stars — Python — archived (read-only): A bot that makes profit of the best crypto Arbitrage opportunities on the market
- **[ryankrumenacker/sports-betting-arbitrage-project](https://github.com/ryankrumenacker/sports-betting-arbitrage-project)** — 204 stars — Jupyter Notebook: statistical-arbitrage-for-sports-betting
- **[bigmacman1129/btc-arbitrage-trading-bot](https://github.com/bigmacman1129/btc-arbitrage-trading-bot)** — 173 stars — JavaScript: Python-based crypto arbitrage bot that monitors price spreads across exchanges, sends alerts, and supports automated trading based on configurable thresholds.
- **[kanekoshoyu/kucoin_arbitrage](https://github.com/kanekoshoyu/kucoin_arbitrage)** — 155 stars — Rust: KuCoin Cyclic Arbitrage, using Event-Driven Async Rust
- **[bigmacman1129/crypto-arbitrage](https://github.com/bigmacman1129/crypto-arbitrage)** — 148 stars — Python: Crypto arbitrage trading bot that detects triangular and cross-exchange opportunities, executes trades automatically, and includes backtested strategies with high historical returns.
- **[cjthoma1/flashswap-arbitrage-avalanche](https://github.com/cjthoma1/flashswap-arbitrage-avalanche)** — 142 stars — TypeScript: Crypto arbitrage bot that searches multiple token exchanges on the Avalanche blockchain for a significant difference in price between two tokens. Once a arbitrage opportunity is found the bot will execute the simultan...
- **[kruglov-dmitry/crypto_crawler](https://github.com/kruglov-dmitry/crypto_crawler)** — 128 stars — Python: Crypto arbitrage bot
- **[OnlyF0uR/EVM-Arbitrage-Bot](https://github.com/OnlyF0uR/EVM-Arbitrage-Bot)** — 107 stars — JavaScript: Fetch and compare SPOT(/SWAP) prices from decentralized exchanges like Uniswap, Sushiswap, and others.
- **[MfaXyz/Crypto-Triangular-Arbitrage-Bot](https://github.com/MfaXyz/Crypto-Triangular-Arbitrage-Bot)** — 103 stars — Python: The algorithm to calculate Triangular Arbitrage with depth on Centralised exchanges.
- **[skellet0r/arbie](https://github.com/skellet0r/arbie)** — 85 stars — Python: Flash Loan Arbitrage Bot targetting Curve.fi Crypto Pool v1
- **[joepegler/Cryptocurrency-Arbitrage-Bot](https://github.com/joepegler/Cryptocurrency-Arbitrage-Bot)** — 83 stars — JavaScript: Cryptocurrency Arbitrage Bot
- **[Blue-Universe/Time-Series-Analysis-Statistical-Arbitrage](https://github.com/Blue-Universe/Time-Series-Analysis-Statistical-Arbitrage)** — 75 stars — Jupyter Notebook: This project used GARCH type models to estimate volatility and used delta hedging method to make a profit.
- **[cjthoma1/flash-loan-bot](https://github.com/cjthoma1/flash-loan-bot)** — 74 stars — Solidity: Crypto arbitrage bot that searches for a significant difference in crypto prices on a number of exchanges and will execute the simultaneous buying and selling of said crypto through a flash loan provided by the AAVE F...
- **[Vikrammel/EasyTrage](https://github.com/Vikrammel/EasyTrage)** — 65 stars — JavaScript: crypto arbitrage bot in MERN
- **[joaop21/hermes](https://github.com/joaop21/hermes)** — 60 stars — Elixir: Bot for cryptocurrency arbitrage
- **[CleBeert/crypto-trading-bot-eth-bnb](https://github.com/CleBeert/crypto-trading-bot-eth-bnb)** — 56 stars — JavaScript: crypto-trading-bot
- **[kurupjayesh/Statistical-Arbitrage-using-Pairs-Trading](https://github.com/kurupjayesh/Statistical-Arbitrage-using-Pairs-Trading)** — 49 stars — Python: 
- **[DiffBit/CryptoTradePlatform](https://github.com/DiffBit/CryptoTradePlatform)** — 47 stars — JavaScript: A Machine Readable Cryptocurrency arbitrage bot
- **[t0mm4rx/binance_crypto_arbitrage](https://github.com/t0mm4rx/binance_crypto_arbitrage)** — 46 stars — Python: A Python bot that watches Binance crypto markets and executes arbitrage
- **[StasonNoder/crypto-arbitrage-bot](https://github.com/StasonNoder/crypto-arbitrage-bot)** — 46 stars: An automated cryptocurrency arbitrage bot that finds price differences across multiple exchanges and executes profitable trades.
- **[philipperemy/Statistical-Arbitrage](https://github.com/philipperemy/Statistical-Arbitrage)** — 36 stars — MATLAB: Using Particle Markov Chain Monte Carlo
- **[Gaochenyin/Quantative-Trading-Models](https://github.com/Gaochenyin/Quantative-Trading-Models)** — 28 stars — Jupyter Notebook: These are trading results and arbitrage models from Southern China Center for Statistical Science (SC2S2), Sun Yat-sen University

## ML & RL trading (53)

Machine-learning prediction and reinforcement-learning trading agents and frameworks.

- **[stefan-jansen/machine-learning-for-trading](https://github.com/stefan-jansen/machine-learning-for-trading)** — 21,623 stars — Jupyter Notebook: Code for Machine Learning for Trading, 3rd edition — from data sourcing to live execution.
- **[AI4Finance-Foundation/FinRL](https://github.com/AI4Finance-Foundation/FinRL)** — 16,849 stars — Jupyter Notebook: FinRL®: Financial Reinforcement Learning.
- **[huseinzol05/Stock-Prediction-Models](https://github.com/huseinzol05/Stock-Prediction-Models)** — 9,511 stars — Jupyter Notebook — archived (read-only): Gathers machine learning and deep learning models for Stock forecasting including trading bots and simulations
- **[tensortrade-org/tensortrade](https://github.com/tensortrade-org/tensortrade)** — 7,253 stars — Python: An open source reinforcement learning framework for training, evaluating, and deploying robust trading agents.
- **[TradeMaster-NTU/TradeMaster](https://github.com/TradeMaster-NTU/TradeMaster)** — 3,117 stars — Jupyter Notebook: TradeMaster is an open-source platform for quantitative trading empowered by reinforcement learning
- **[robertmartin8/MachineLearningStocks](https://github.com/robertmartin8/MachineLearningStocks)** — 1,966 stars — Python: Using python and scikit-learn to make stock predictions
- **[PacktPublishing/Hands-On-Machine-Learning-for-Algorithmic-Trading](https://github.com/PacktPublishing/Hands-On-Machine-Learning-for-Algorithmic-Trading)** — 1,944 stars — Jupyter Notebook: Hands-On Machine Learning for Algorithmic Trading, published by Packt
- **[Yvictor/TradingGym](https://github.com/Yvictor/TradingGym)** — 1,925 stars — Python: Trading and Backtesting environment for training reinforcement learning agent or simple rule base algo.
- **[asavinov/intelligent-trading-bot](https://github.com/asavinov/intelligent-trading-bot)** — 1,886 stars — Python: Intelligent Trading Bot: Automatically generating signals and trading based on machine learning and feature engineering
- **[tradytics/surpriver](https://github.com/tradytics/surpriver)** — 1,884 stars — Python: Find big moving stocks before they move using machine learning and anomaly detection
- **[notadamking/RLTrader](https://github.com/notadamking/RLTrader)** — 1,867 stars — Python: A cryptocurrency trading environment using deep reinforcement learning and OpenAI's gym
- **[LastAncientOne/Deep_Learning_Machine_Learning_Stock](https://github.com/LastAncientOne/Deep_Learning_Machine_Learning_Stock)** — 1,789 stars — Jupyter Notebook: Deep Learning and Machine Learning stocks represent promising opportunities for both long-term and short-term investors and traders.
- **[PacktPublishing/Machine-Learning-for-Algorithmic-Trading-Second-Edition_Original](https://github.com/PacktPublishing/Machine-Learning-for-Algorithmic-Trading-Second-Edition_Original)** — 1,645 stars — Jupyter Notebook: Machine Learning for Algorithmic Trading, Second Edition - published by Packt
- **[deependersingla/deep_trader](https://github.com/deependersingla/deep_trader)** — 1,508 stars — Python: This project uses reinforcement learning on stock market and agent tries to learn trading. The goal is to check if the agent can learn to read tape. The project is dedicated to hero in life great Jesse Livermore.
- **[Rachnog/Deep-Trading](https://github.com/Rachnog/Deep-Trading)** — 1,463 stars — OpenEdge ABL: Algorithmic trading with deep learning experiments
- **[Ceruleanacg/Personae](https://github.com/Ceruleanacg/Personae)** — 1,408 stars — Python: Personae is a repo of implements and environment of Deep Reinforcement Learning & Supervised Learning for Quantitative Trading.
- **[sadighian/crypto-rl](https://github.com/sadighian/crypto-rl)** — 966 stars — Python: Deep Reinforcement Learning toolkit: record and replay cryptocurrency limit order book data & train a DDQN agent
- **[samre12/deep-trading-agent](https://github.com/samre12/deep-trading-agent)** — 796 stars — Python: Deep Reinforcement Learning based Trading Agent for Bitcoin
- **[hackthemarket/gym-trading](https://github.com/hackthemarket/gym-trading)** — 712 stars — Jupyter Notebook: Environment for reinforcement-learning algorithmic trading models
- **[Albert-Z-Guo/Deep-Reinforcement-Stock-Trading](https://github.com/Albert-Z-Guo/Deep-Reinforcement-Stock-Trading)** — 688 stars — Jupyter Notebook: A light-weight deep reinforcement learning framework for portfolio management. This project explores the possibility of applying deep reinforcement learning algorithms to stock trading in a highly modular and scalable...
- **[yacoubb/stock-trading-ml](https://github.com/yacoubb/stock-trading-ml)** — 669 stars — Python: A stock trading bot that uses machine learning to make price predictions.
- **[thedimlebowski/Trading-Gym](https://github.com/thedimlebowski/Trading-Gym)** — 568 stars — Python: Trading Gym is an open source project for the development of reinforcement learning algorithms in the context of trading.
- **[ucaiado/QLearning_Trading](https://github.com/ucaiado/QLearning_Trading)** — 517 stars — Jupyter Notebook: Learning to trade under the reinforcement learning framework
- **[evgps/a3c_trading](https://github.com/evgps/a3c_trading)** — 437 stars — Jupyter Notebook: Trading with recurrent actor-critic reinforcement learning
- **[TorchTrade/torchtrade](https://github.com/TorchTrade/torchtrade)** — 429 stars — Python: Modular reinforcement learning framework for algorithmic trading
- **[pythonlessons/RL-Bitcoin-trading-bot](https://github.com/pythonlessons/RL-Bitcoin-trading-bot)** — 417 stars — Python: Trying to create Reinforcement Learning powered Bitcoin trading bot
- **[golsun/deep-RL-trading](https://github.com/golsun/deep-RL-trading)** — 360 stars — Python: playing idealized trading games with deep reinforcement learning
- **[yuriak/RLQuant](https://github.com/yuriak/RLQuant)** — 359 stars — Jupyter Notebook: Applying Reinforcement Learning in Quantitative Trading
- **[gsyyysg/StockFormer](https://github.com/gsyyysg/StockFormer)** — 358 stars — Python: PyTorch implementation for Paper "StockFormer: Learning Hybrid Trading Machines with Predictive Coding".
- **[benstaf/FinRL_DeepSeek](https://github.com/benstaf/FinRL_DeepSeek)** — 331 stars — Jupyter Notebook: Code for the paper "FinRL-DeepSeek: LLM-Infused Risk-Sensitive Reinforcement Learning for Trading Agents" arXiv:2502.07393
- **[javifalces/HFTFramework](https://github.com/javifalces/HFTFramework)** — 309 stars — Jupyter Notebook: HFTFramework utilized for research on " A reinforcement learning approach to improve the performance of the Avellaneda-Stoikov market-making algorithm "
- **[sachink2010/AutomatedStockTrading-DeepQ-Learning](https://github.com/sachink2010/AutomatedStockTrading-DeepQ-Learning)** — 302 stars — Jupyter Notebook: Every day, millions of traders around the world are trying to make money by trading stocks. These days, physical traders are also being replaced by automated trading robots. Algorithmic trading market has experienced...
- **[ucaiado/rl_trading](https://github.com/ucaiado/rl_trading)** — 290 stars — Python: An environment to high-frequency trading agents under reinforcement learning
- **[LiamConnell/deep-algotrading](https://github.com/LiamConnell/deep-algotrading)** — 247 stars — Jupyter Notebook: A resource for learning about deep learning techniques from regression to LSTM and Reinforcement Learning using financial data and the fitness functions of algorithmic trading
- **[hugocen/freqtrade-gym](https://github.com/hugocen/freqtrade-gym)** — 229 stars — Python: A customized gym environment for developing and comparing reinforcement learning algorithms in crypto trading.
- **[kennedyCzar/STOCK-RETURN-PREDICTION-USING-KNN-SVM-GUASSIAN-PROCESS-ADABOOST-TREE-REGRESSION-AND-QDA](https://github.com/kennedyCzar/STOCK-RETURN-PREDICTION-USING-KNN-SVM-GUASSIAN-PROCESS-ADABOOST-TREE-REGRESSION-AND-QDA)** — 137 stars — Jupyter Notebook: Forecast stock prices using machine learning approach. A time series analysis. Employ the Use of Predictive Modeling in Machine Learning to Forecast Stock Return. Approach Used by Hedge Funds to Select Tradeable Stocks
- **[SravB/Algorithmic-Trading](https://github.com/SravB/Algorithmic-Trading)** — 118 stars — Python: Algorithmic trading using machine learning.
- **[theanh97/Deep-Reinforcement-Learning-with-Stock-Trading](https://github.com/theanh97/Deep-Reinforcement-Learning-with-Stock-Trading)** — 102 stars — Jupyter Notebook: This project uses Deep Reinforcement Learning (DRL) to develop and evaluate stock trading strategies. By implementing agents like PPO, A2C, DDPG, SAC, and TD3 in a realistic trading environment with transaction costs,...
- **[jcwill415/Stock_Market_Data_Analysis](https://github.com/jcwill415/Stock_Market_Data_Analysis)** — 99 stars — Jupyter Notebook: Scrape, analyze & visualize stock market data for the S&P500 using Python. Build a basic trading strategy using machine learning to assess company performance and determine buy, sell, hold. Read me & instructions avai...
- **[TimRivoli/Stock-Price-Trade-Analyzer](https://github.com/TimRivoli/Stock-Price-Trade-Analyzer)** — 78 stars — Python: This is a Python 3 project for analyzing stock prices and methods of stock trading. It uses native Python tools and Google TensorFlow machine learning.
- **[lnmangione/Stock-Bot](https://github.com/lnmangione/Stock-Bot)** — 63 stars — Java: Stock Trading Bot using Machine Learning
- **[Amey-Thakur/OPTIMIZING-STOCK-TRADING-STRATEGY-WITH-REINFORCEMENT-LEARNING](https://github.com/Amey-Thakur/OPTIMIZING-STOCK-TRADING-STRATEGY-WITH-REINFORCEMENT-LEARNING)** — 58 stars — Python: Machine Learning Project to Optimize Stock Trading Strategies Using Reinforcement Learning.
- **[hahnicity/pytrader](https://github.com/hahnicity/pytrader)** — 52 stars — Jupyter Notebook: An open source trading bot to leverage machine learning in the stock market
- **[shivamakhauri04/TradingBot](https://github.com/shivamakhauri04/TradingBot)** — 51 stars — Jupyter Notebook: Stock price prediction and automated trading using Deep Reinforcement Learning and Machine Learning
- **[xljiang/stock-market-simulator](https://github.com/xljiang/stock-market-simulator)** — 51 stars — Python: A stock market simulator that generate best trading strategy using machine learning tools.
- **[deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports](https://github.com/deepak223098/Long-Term-Stock-Price-Growth-Prediction-using-NLP-on-10-K-Financial-Reports)** — 49 stars — HTML: A 10-K FInancial Report is a comprehensive report which must be filed annually by all publicly traded companies about its financial performance. These reports are filed to the US Securities Exchange Commission (SEC)....
- **[jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis](https://github.com/jaiminsanghvi/Analysing-Stock-Market-Movements-Using-Twitter-Sentiment-Analysis)** — 48 stars — Python: Stock market prediction is the method of trying to determine the future value of publically listed company stock traded on an exchange. The successful prediction of a stock's future price could yield significant profi...
- **[kknet/AlgorithmicTrading](https://github.com/kknet/AlgorithmicTrading)** — 44 stars — Python: Select a supervised algorithm that can predict stock prices of historical data based on the predictors (statistical indicators). Accordingly formulate a trading strategy based on predicted values to generate orders on...
- **[hklchung/StockPricePredictor](https://github.com/hklchung/StockPricePredictor)** — 39 stars — Jupyter Notebook: An S&P500 Stock Index Movement Predictor built with Machine Learning models
- **[sushantsbelapurkar/Stock-Portfolio-Robo-Advisor](https://github.com/sushantsbelapurkar/Stock-Portfolio-Robo-Advisor)** — 19 stars — Jupyter Notebook: Stock Portfolio Robo advisor is the automatic stock portfolio construction and rebalancing software created. Using listed companies' financial statements, we constructed a portfolio based on value investment and growt...
- **[jsgaston/Mastering-Algorithmic-Trading-with-Deep-Learning-A-Comprehensive-Guide-to-LSTM-Based-Trading-Systems](https://github.com/jsgaston/Mastering-Algorithmic-Trading-with-Deep-Learning-A-Comprehensive-Guide-to-LSTM-Based-Trading-Systems)** — 14 stars — Python: presents an in-depth exploration of a cutting-edge trading architecture that combines the predictive power of Long Short-Term Memory (LSTM) neural networks with the precise execution capabilities of MetaTrader 5.
- **[Dahshan228/FTMO-TradingBot](https://github.com/Dahshan228/FTMO-TradingBot)** — 8 stars — Python: Multi-strategy algorithmic trading bot for FTMO prop accounts - LightGBM equity signal + BTC/forex trend following with full FTMO compliance. Backtested 2015-2025, 1600+ tests, live-validated on demo.
- **[Anshupriya2694/Portfolio-Optimization-Methods](https://github.com/Anshupriya2694/Portfolio-Optimization-Methods)** — 5 stars — Jupyter Notebook: This project used Neural Networks and ARIMA to perform portfolio rebalancing. Their performance is measured using MSE and the more efficient algorithm is used to perform realtime portfolio rebalncing.

## Options & volatility (26)

Options strategy implementations, volatility trading and options backtesting libraries.

- **[goldspanlabs/optopsy](https://github.com/goldspanlabs/optopsy)** — 1,506 stars — Python: A nimble options research and backtesting library for Python
- **[AlgoTraders/stock-analysis-engine](https://github.com/AlgoTraders/stock-analysis-engine)** — 1,246 stars — Jupyter Notebook: Backtest 1000s of minute-by-minute trading algorithms for training AI with automated pricing data from: IEX, Tradier and FinViz. Datasets and trading performance automatically published to S3 for building AI training...
- **[JerBouma/AlgorithmicTrading](https://github.com/JerBouma/AlgorithmicTrading)** — 1,104 stars — Jupyter Notebook — archived (read-only): This repository contains three ways to obtain arbitrage which are Dual Listing, Options and Statistical Arbitrage. These are projects in collaboration with Optiver and have been peer-reviewed by staff members of Optiver.
- **[PyPatel/Options-Trading-Strategies-in-Python](https://github.com/PyPatel/Options-Trading-Strategies-in-Python)** — 1,095 stars — Python: Developing Options Trading Strategies using Technical Indicators and Quantitative Methods
- **[rgaveiga/optionlab](https://github.com/rgaveiga/optionlab)** — 582 stars — Python: A Python library for evaluating option trading strategies.
- **[mcf-long-short/ibkr-options-volatility-trading](https://github.com/mcf-long-short/ibkr-options-volatility-trading)** — 376 stars — Python: Volatility trading using Long and Short Straddle options strategies on Interactive Broker using Yahoo Finance and TWS API
- **[fintechees/Expert-Advisor-Studio](https://github.com/fintechees/Expert-Advisor-Studio)** — 364 stars — JavaScript: Expert Advisors, Browser-based EA, Free Crypto / Forex Historical DATA API plugins, EA Codes Generator, Market Maker Bot, Crypto Options Plugin, DEX / Payment Gateway Plugins, Binance API, FIX API related. Fintechee m...
- **[deshwalmahesh/NSE-Stock-Scanner](https://github.com/deshwalmahesh/NSE-Stock-Scanner)** — 351 stars — Jupyter Notebook: National Stock Exchange (NSE), India based Stock screener program. Supports Live Data, Swing / Momentum Trading, Intraday Trading, Connect to online brokers as Zerodha Kite, Risk Management, Emotion Control, Screening...
- **[bradleyboyuyang/Statistical-Arbitrage](https://github.com/bradleyboyuyang/Statistical-Arbitrage)** — 287 stars — Jupyter Notebook: High-frequency statistical arbitrage
- **[randomwalkhan/Short-Term-Reversal-Strategy](https://github.com/randomwalkhan/Short-Term-Reversal-Strategy)** — 262 stars — Jupyter Notebook: Python-based quant trading research project for short-term reversal option setups, universe selection, staged-entry backtesting, and live paper trading.
- **[joaquinbejar/OptionStratLib](https://github.com/joaquinbejar/OptionStratLib)** — 261 stars — Rust: OptionStratLib is a comprehensive Rust library for options trading and strategy development across multiple asset classes.
- **[OptionsnPython/Option-strategies-backtesting-in-Python](https://github.com/OptionsnPython/Option-strategies-backtesting-in-Python)** — 179 stars — Jupyter Notebook: Python codes used in book 'Option Greeks Strategies & Backtesting in Python'
- **[cutemarkets/cutebacktests](https://github.com/cutemarkets/cutebacktests)** — 164 stars — Python: Backtesting framework for modern option strategies
- **[alpacahq/gamma-scalping](https://github.com/alpacahq/gamma-scalping)** — 162 stars — Python: Runnable algo template for gamma scalping options trading strategy
- **[alpacahq/options-wheel](https://github.com/alpacahq/options-wheel)** — 135 stars — Python: Runnable algo template for trading the Options Wheel strategy
- **[aaryansinha16/AI-trader](https://github.com/aaryansinha16/AI-trader)** — 80 stars — Python: AI-powered intraday options trading system for NSE F&O. Dual ML models + institutional flow analysis + regime-adaptive strategies. Python/TimescaleDB.
- **[Talha-Tariq/poptions](https://github.com/Talha-Tariq/poptions)** — 68 stars — Python: Custom Python code for calculating the Probability of Profit (POP) for options trading strategies using Monte Carlo Simulations. The Monte Carlo Simulation runs thousands of individual stock price simulations and uses...
- **[m12t/bsm-time-machine](https://github.com/m12t/bsm-time-machine)** — 65 stars — Jupyter Notebook: A Black-Scholes-based options backtesting simulator
- **[theoptionlab/theoptionlab](https://github.com/theoptionlab/theoptionlab)** — 64 stars — HTML: The Option Lab is an automated backtesting framework for option trading strategies. The results generated by the code are visualised on our website.
- **[marksantiago290/OrderFlowBot-NinjaTrading](https://github.com/marksantiago290/OrderFlowBot-NinjaTrading)** — 63 stars — C#: OrderFlowBot is a bot used for trading order flow with a selected ATM (Automated Trading Mechanism) strategy. It is designed to assist with entries in a semi-automated trading approach, but also offers a fully automat...
- **[AlexShakaev/backtesting_and_algotrading_options_with_Interactive_Brokers_API](https://github.com/AlexShakaev/backtesting_and_algotrading_options_with_Interactive_Brokers_API)** — 60 stars — Jupyter Notebook: backtesting and algotrading options using Interactive Brokers API (native python api)
- **[anthonymakarewicz/volatility-trading](https://github.com/anthonymakarewicz/volatility-trading)** — 46 stars — Jupyter Notebook: Systematic Volatility Research and Backtesting for equity options
- **[vavarb/vavabot_options_strategy](https://github.com/vavarb/vavabot_options_strategy)** — 37 stars — Python: Deribit bot to run options strategy orders with different triggers and targets. You can set strategy cost to execute orders, this can be used to start trade, stop loss or stop gain. It supports strategies with up to 4...
- **[Skyboi94/Quant-Trading-Projects](https://github.com/Skyboi94/Quant-Trading-Projects)** — 21 stars — Jupyter Notebook: 11 hands-on quant trading projects in Python: backtesting, options pricing, stat arb, ML prediction, and execution algorithms.
- **[pixelwhiz/tasty-schwab-trader-BE](https://github.com/pixelwhiz/tasty-schwab-trader-BE)** — 14 stars — Python: Multi-strategy algorithmic trading platform designed for institutional-grade automated execution across futures, equities, and options markets. Built with Python and Flask, this system provides real-time market analys...
- **[marketcalls/openalgo-execution-skills](https://github.com/marketcalls/openalgo-execution-skills)** — 9 stars — Python: Dual-mode (backtest + live) algorithmic trading strategy skills for OpenAlgo. VectorBT backtests, OpenAlgo live execution, real-world Indian costs and slippage, options multi-leg execution, ML strategies, portfolio ri...

## Grid & DCA (13)

Grid-trading and dollar-cost-averaging bot implementations.

- **[Open-Trader/opentrader](https://github.com/Open-Trader/opentrader)** — 2,912 stars — TypeScript: Open-source crypto trading bot | DCA & GRID strategies | UI | Star to support the project!
- **[51bitquant/binance_grid_trader](https://github.com/51bitquant/binance_grid_trader)** — 987 stars — Python: Grid strategy bot for Binance Spot and Futures; supports any Binance pair.
- **[PyWaves/BlackBot](https://github.com/PyWaves/BlackBot)** — 256 stars — Python: Grid Trading Bot
- **[DogsTailFarmer/martin-binance](https://github.com/DogsTailFarmer/martin-binance)** — 228 stars — Python: Free trading system for crypto exchanges SPOT market. Adaptive customizable reverse grid strategy based on martingale.
- **[cyberjunky/3commas-cyber-bots](https://github.com/cyberjunky/3commas-cyber-bots)** — 218 stars — Python: 3Commas bot helpers, AltRank, GalaxyScore, Watchlists, Auto-Compound, TrailingStopLoss, TakeProfitIncrement
- **[HenrisonTao/ftx_grid_trading_bot](https://github.com/HenrisonTao/ftx_grid_trading_bot)** — 170 stars — Python: The program was designed to run the Grid Trading Strategy in FTX exchange
- **[jordantete/grid_trading_bot](https://github.com/jordantete/grid_trading_bot)** — 152 stars — Python: Open-source cryptocurrency trading bot designed to perform grid trading strategies using historical data for backtesting
- **[tiansin/docker_grid_trader](https://github.com/tiansin/docker_grid_trader)** — 110 stars — Python: Grid trading strategy and bot for the Binance exchange.
- **[0xTaoDev/jupiter-python-cli](https://github.com/0xTaoDev/jupiter-python-cli)** — 106 stars — Python: Jupiter Python CLI is a Command Line Interface (CLI) where you can use Jupiter features including a Sniper Bot.
- **[CyberPunkMetalHead/binance-dca-crypto-trading-bot](https://github.com/CyberPunkMetalHead/binance-dca-crypto-trading-bot)** — 55 stars — Python: This is a DCA crypto trading bot built for Binance written in Python. It works by allowing you to DCA at an interval of your choosing and reports back on your average buy price as well as a chart consisting of each DCA.
- **[tulihub/PT-MoonBot](https://github.com/tulihub/PT-MoonBot)** — 46 stars — JavaScript: MoonBot is the ultimate addon for mobile control Profit Trailer!
- **[piebotai/bot](https://github.com/piebotai/bot)** — 36 stars — Python — archived (read-only): PieBot is a DCA (Dollar Cost Averaging) cryptocurrency trading bot, built with Python, for the Crypto.com Exchange
- **[JozefJarosciak/3commas-python-selfadjusting-dca-bot](https://github.com/JozefJarosciak/3commas-python-selfadjusting-dca-bot)** — 29 stars — Python: Self-adjusting, auto-compounding multi-pair DCA crypto trading bot using Python, AWS Lambda & 3Commas API

## Execution algorithms (5)

Optimal execution research implementations (scheduling, implementation shortfall, market impact).

- **[sduprey/optimal_transaction_execution](https://github.com/sduprey/optimal_transaction_execution)** — 35 stars — Matlab: This entry contains two topics The first item is entirely based on the following paper: http://sfb649.wiwi.hu-berlin.de/papers/pdf/SFB649DP2011-056.pdf It contains 2 MATLAB demonstrating script : DATA_preprocessing.m...
- **[yc3566/Optimal-Trade-Execution-Algorithm-Market-Impact-Model](https://github.com/yc3566/Optimal-Trade-Execution-Algorithm-Market-Impact-Model)** — 19 stars — Jupyter Notebook: 2 algorithms of optimal trade execution: 1) Dynamic Programming 2) Frank-Wolfe Algorithm (Python & C++)
- **[Weichong515/Algo-Trading-14](https://github.com/Weichong515/Algo-Trading-14)** — 16 stars — Jupyter Notebook: Momentum following strategies and optimal execution cost upon Implement Shortfall algorithm
- **[NikhilSehgal123/coinbase-execution-algorithm](https://github.com/NikhilSehgal123/coinbase-execution-algorithm)** — 14 stars — Python: An algorithm that intelligently executes a crypto order over time via Coinbase
- **[bhardwajshivam/Optimizing-Order-Execution-Strategy-for-HFT](https://github.com/bhardwajshivam/Optimizing-Order-Execution-Strategy-for-HFT)** — 7 stars — Jupyter Notebook: I will be working with a simulated market dataset and focus on optimizing order execution strategies for a high-frequency trading (HFT). The goal is to design algorithms that can execute trades with minimal market imp...

## Prediction markets (6)

Trading bots and market makers for prediction markets (Polymarket, Kalshi, Manifold).

- **[evan-kolberg/prediction-market-backtesting](https://github.com/evan-kolberg/prediction-market-backtesting)** — 1,258 stars — Python: An extension for Nautilus Trader
- **[Polytutor-AI-Labs/polymarket-trading-bot-strategy](https://github.com/Polytutor-AI-Labs/polymarket-trading-bot-strategy)** — 151 stars: PolyTutor overview of Polymarket trading bots: strategy guides (copy trading, market making, dump-hedge, last-seconds, sports), case studies, and engineering notes. Smart trading. Real edge.
- **[Drakkar-Software/OctoBot-Prediction-Market](https://github.com/Drakkar-Software/OctoBot-Prediction-Market)** — 121 stars — Python: Polymarket open source trading bot to automate copy trading and arbitrage strategies on crypto prediction markets for free, with simple interface.
- **[lalouannabel0072/kalshi-trading-bot](https://github.com/lalouannabel0072/kalshi-trading-bot)** — 54 stars — Python: Kalshi trading bot for prediction markets — browse event contracts and watch the order book in real time, run market making and momentum/reversion strategies with paper trading, risk limits and position sizing. Educat...
- **[else24/kalshi-market-bot](https://github.com/else24/kalshi-market-bot)** — 37 stars — Python: Kalshi trading bot for prediction markets — browse event contracts and watch the order book in real time, run market making and momentum/reversion strategies with paper trading, risk limits and position sizing. Educat...
- **[emmanuelwestra/Polymarket-Trading-Bot](https://github.com/emmanuelwestra/Polymarket-Trading-Bot)** — 35 stars — TypeScript: The most advanced open-source Polymarket trading bot. 7 automated strategies (arbitrage, convergence, market making, momentum, AI forecast), whale tracker with copy-trade simulator, real-time dashboard, parallel marke...

## Other strategies (41)

Multi-strategy collections, educational implementations and courses with runnable strategy code.

- **[je-suis-tm/quant-trading](https://github.com/je-suis-tm/quant-trading)** — 11,150 stars — Python: Python quantitative trading strategies including VIX Calculator, Pattern Recognition, Commodity Trading Advisor, Monte Carlo, Options Straddle, Shooting Star, London Breakout, Heikin-Ashi, Pair Trading, RSI, Bollinger...
- **[chrisconlan/algorithmic-trading-with-python](https://github.com/chrisconlan/algorithmic-trading-with-python)** — 3,518 stars — Python: Source code for Algorithmic Trading with Python (2020) by Chris Conlan
- **[nickmccullum/algorithmic-trading-python](https://github.com/nickmccullum/algorithmic-trading-python)** — 2,902 stars — Jupyter Notebook: The repository for freeCodeCamp's YouTube course, Algorithmic Trading in Python
- **[joshyattridge/smart-money-concepts](https://github.com/joshyattridge/smart-money-concepts)** — 2,122 stars — Python: Discover our Python package designed for algorithmic trading. It brings ICT's smart money concepts to Python, offering a range of indicators for your algorithmic trading strategies.
- **[LongOnly/Quantitative-Notebooks](https://github.com/LongOnly/Quantitative-Notebooks)** — 1,400 stars — Jupyter Notebook — archived (read-only): Educational notebooks on quantitative finance, algorithmic trading, financial modelling and investment strategy
- **[PacktPublishing/Python-for-Algorithmic-Trading-Cookbook](https://github.com/PacktPublishing/Python-for-Algorithmic-Trading-Cookbook)** — 1,252 stars — Jupyter Notebook: Python for Algorithmic Trading Cookbook, published by Packt
- **[everget/tradingview-pinescript-indicators](https://github.com/everget/tradingview-pinescript-indicators)** — 947 stars: A collection of various technical indicators implemented in Pine Script for the TradingView platform
- **[EarnForex/PositionSizer](https://github.com/EarnForex/PositionSizer)** — 608 stars — MQL5: Calculate your position size based on the risk and account size and execute your trades with this free MetaTrader expert advisor.
- **[PythonProgramming/Pattern-Recognition-for-Forex-Trading](https://github.com/PythonProgramming/Pattern-Recognition-for-Forex-Trading)** — 252 stars — Python: http://pythonprogramming.net/machine-learning-pattern-recognition-algorithmic-forex-stock-trading/
- **[Joaopeuko/Mql5-Python-Integration](https://github.com/Joaopeuko/Mql5-Python-Integration)** — 215 stars — Python: I created this library to streamline the process of developing an Expert Advisor in MQL5. While MQL5 development can be complex, the same task is more straightforward in Python.
- **[3aLaee/xauusd-trading-bot](https://github.com/3aLaee/xauusd-trading-bot)** — 199 stars — Python: Trading bot scalper for the XAU/USD (Gold to US Dollar) pair
- **[matthewkastor/Metatrader](https://github.com/matthewkastor/Metatrader)** — 164 stars — HTML: Expert advisors, scripts, indicators and code libraries for Metatrader.
- **[grantcolley/tradeview](https://github.com/grantcolley/tradeview)** — 145 stars — C#: A .NET 10 WPF client and ASP.NET Core Web API platform for trading and running crypto currency pair strategies.
- **[ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha](https://github.com/ethics/Automatic-Trading-using-Artificial-Intelligence-machine-Learning-on-Indian-Stock-Market-_Zerodha)** — 76 stars — Jupyter Notebook: Online trading using Artificial Intelligence Machine leaning with python on Indian Stock Market
- **[pipbolt/experts](https://github.com/pipbolt/experts)** — 62 stars — MQL5: Expert Advisors (aka Forex Trading Robots) for Metatrader 5
- **[EarnForex/News-Trader](https://github.com/EarnForex/News-Trader)** — 57 stars — MQL5: News Trader is a customizable expert advisor to trade high-volatility news releases based on the trader's inputs.
- **[amstrdm/mlm-trend-following](https://github.com/amstrdm/mlm-trend-following)** — 47 stars — Python: Automated Python implementation of a Mount Lucas Management (MLM) style trend-following strategy for futures using the IBKR API (ib_insync). Calculates 200-day MA signals on Continuous Futures, filters trades by volat...
- **[JeffreyHyer/grandmaster](https://github.com/JeffreyHyer/grandmaster)** — 46 stars — JavaScript: Backtesting and execution of algorithmic trading strategies in Node.js
- **[grananqvist/Machine-Learning-Trading-Strategies](https://github.com/grananqvist/Machine-Learning-Trading-Strategies)** — 45 stars — Jupyter Notebook: Stock and Forex market prediction using ML and time-series modelling
- **[NadirAliOfficial/STAR-EA-v11.20](https://github.com/NadirAliOfficial/STAR-EA-v11.20)** — 45 stars — MQL5: ICT-based MT5 Expert Advisor with 12-scenario market classification engine — Order Blocks, FVG, OTE, Liquidity, CRT, Judas Swing, Silver Bullet, AMD
- **[theriley106/TheWSBIndex](https://github.com/theriley106/TheWSBIndex)** — 43 stars — JavaScript: Algorithmic trading strategy to predict market volatility from the post/comment frequency on a popular trading forum
- **[Storkle/clj-forex](https://github.com/Storkle/clj-forex)** — 42 stars — Clojure: Create clojure expert advisors for forex with various backends (metatrader, ...)
- **[lightningfast66/forex-mt5-viper-strategy](https://github.com/lightningfast66/forex-mt5-viper-strategy)** — 28 stars: A high-precision trend-following indicator and trading strategy for MetaTrader 5 (MT5). Features dynamic volatility filtering, no-repaint signals, and automated risk management.
- **[paaak/TradeConnector](https://github.com/paaak/TradeConnector)** — 24 stars — TypeScript: Cloud-native, algorithmic trade execution platform, harnessing data-driven insights for adaptive, centralized market access Portal.
- **[WQU-MScFE-Capstone-MGS/qc-tick-data-strategies](https://github.com/WQU-MScFE-Capstone-MGS/qc-tick-data-strategies)** — 23 stars — Python: Repo for the Tick Based Trend Following strategies written for the QuantConnect platform
- **[Marcnuth/AutoQuant](https://github.com/Marcnuth/AutoQuant)** — 22 stars — JavaScript: AutoQuant is an out-of-the-box quantitative investment platform.
- **[cjjeakle/rebalance-calc](https://github.com/cjjeakle/rebalance-calc)** — 20 stars — TypeScript: A Portfolio Rebalancing and Tax Optimization Calculator
- **[tranceitionalMynd/spy-reaper](https://github.com/tranceitionalMynd/spy-reaper)** — 17 stars — Python: Algorithmic trading application for use with Interactive Brokers
- **[BrockStar3540/mr-scrooge-v6](https://github.com/BrockStar3540/mr-scrooge-v6)** — 12 stars — Python: Open-source algorithmic forex trading bot for OANDA (Python) — cell-based execution, wide-stop ratchet exits, a full backtesting research program, and a live control-panel dashboard. Strategy-neutral by design; publis...
- **[kalipsot/Hyperliquid-Basic-Trend](https://github.com/kalipsot/Hyperliquid-Basic-Trend)** — 12 stars — Python: basic trend following strategy for the crypto exchange hyperliquid
- **[garroshub/taxrebalance](https://github.com/garroshub/taxrebalance)** — 12 stars — Python: Python toolkit for tax-aware portfolio rebalancing with Canadian ACB, U.S. tax lots, tracking-error constraints, and convex optimization.
- **[LuckyMan22-SuperMan/autonomous_trading_bot](https://github.com/LuckyMan22-SuperMan/autonomous_trading_bot)** — 8 stars — Python: A FastAPI-powered algorithmic trading terminal supporting strategy backtesting (SMA crossover, MACD trend, etc.) and live paper trading execution, with real-time market data via Yahoo Finance for Indian and US equities.
- **[py310/portfolio-rebalance-methods](https://github.com/py310/portfolio-rebalance-methods)** — 8 stars — Python: Portfolio Rebalance Methods: Explore and implement diverse rebalancing strategies, including Static Weights, Risk Parity, and Markowitz, to optimize and fine-tune your investment portfolios.
- **[AnkanMisra/FlexiYield](https://github.com/AnkanMisra/FlexiYield)** — 8 stars — Rust: FlexiYield is a Solana‑native dynamic stablecoin basket that turns USDC/USDT into an auto‑optimized, diversified portfolio. Deposit, mint a basket token, and get rules‑based rebalancing, de‑peg protection, and real‑ti...
- **[0xmurtimus/RiskDiversifier](https://github.com/0xmurtimus/RiskDiversifier)** — 7 stars — Go: Intelligent, data-driven portfolio optimization algorithms harmonize Auto-adjusting portfolio rebalancing, performance monitoring, driving informed decision-making as a transformation catalyst.
- **[topsecretagent007/sol-rebalancing-bot-public](https://github.com/topsecretagent007/sol-rebalancing-bot-public)** — 7 stars — TypeScript: SOL Rebalancing Bot automates portfolio balancing for Solana (SOL) holdings, adjusting asset allocations based on predefined strategies to optimize risk and returns on-chain.
- **[jctags/mse448-trend-following](https://github.com/jctags/mse448-trend-following)** — 6 stars — Python: Trend Following Strategy for an MS&E 448 Project
- **[amoghathalekar/Stochastic-RSI-Trend-Following-Strategy](https://github.com/amoghathalekar/Stochastic-RSI-Trend-Following-Strategy)** — 6 stars — Python: This is a Trend Following strategy that uses RSI and Stochastics(in its purest form) with a trailing stop loss set by Chandelier Exit indicator.
- **[BuzuGit/Trend-Following-Analysis](https://github.com/BuzuGit/Trend-Following-Analysis)** — 6 stars — Jupyter Notebook: Compares Trend Following investment strategy to Buy and Hold.
- **[WISEPLAT/sideways_indicator](https://github.com/WISEPLAT/sideways_indicator)** — 6 stars: The Sideway indicator for TradingView is a powerful tool designed to identify periods of sideways or ranging price action in the market. With its intuitive interface and customizable parameters, traders can easily spo...
- **[sduprey/portfolio_backtesting](https://github.com/sduprey/portfolio_backtesting)** — 6 stars — Matlab: We optimize our portoflio using constrained 130/30 efficient frontier. The definition and backtesting of the strategy is done through a GUI and outputs a dynamic report to highlight the strategy (portfolio composition...

## Supporting tools and unresolved candidates (29)

API wrappers, MCP servers, UIs, plotting addons, screeners and agent toolkits found in the same searches. Most do not themselves implement a trading algorithm; a few describe strategy examples but were not verified.

- **[atilaahmettaner/tradingview-mcp](https://github.com/atilaahmettaner/tradingview-mcp)** — 5,163 stars — Python: MCP server for market analysis in AI assistants: screeners, multi-timeframe technical analysis, backtests and sentiment for crypto, stocks, forex and futures. 39 tools. Self-host free, or use the hosted version, Tradi...
- **[jrothschild33/learn_backtrader](https://github.com/jrothschild33/learn_backtrader)** — 2,288 stars — Python: Systematic Chinese-language tutorial on Backtrader: strategy construction, data structures and backtesting.
- **[alpacahq/alpaca-mcp-server](https://github.com/alpacahq/alpaca-mcp-server)** — 1,048 stars — Python: Alpaca’s official MCP Server lets you trade stocks, ETFs, crypto, and options, run data analysis, and build strategies in plain English directly from your favorite LLM tools and IDEs
- **[Ckend/pythondict-quant](https://github.com/Ckend/pythondict-quant)** — 878 stars — Python: Quant trading examples based on Backtrader.
- **[alpacahq/alpaca-backtrader-api](https://github.com/alpacahq/alpaca-backtrader-api)** — 703 stars — Python: Alpaca Trading API integrated with backtrader
- **[QuantWorld2022/backtrader](https://github.com/QuantWorld2022/backtrader)** — 587 stars — Jupyter Notebook: 
- **[Skinok/backtrader-pyqt-ui](https://github.com/Skinok/backtrader-pyqt-ui)** — 518 stars — Python: Easy to use backtrader UI
- **[verybadsoldier/backtrader_plotting](https://github.com/verybadsoldier/backtrader_plotting)** — 501 stars — Python — archived (read-only): Plotting addon for backtrader to support Bokeh (and maybe more)
- **[okx/agent-trade-kit](https://github.com/okx/agent-trade-kit)** — 492 stars — TypeScript: OKX trading MCP server — connect AI agents to spot, swap, futures, options & grid bots via the Model Context Protocol.
- **[ccxt-rest/ccxt-rest](https://github.com/ccxt-rest/ccxt-rest)** — 486 stars — JavaScript: Open Source Unified REST API of 100+ Crypto Exchange Sites (18k+ docker pulls) - https://ccxt-rest.io/
- **[moss-site/moss-trade-bot-skills](https://github.com/moss-site/moss-trade-bot-skills)** — 390 stars — Python: LLM-powered trading agents that turn plain natural language into a five-pillar strategy: Trend, Mean-Reversion, Momentum, Volume, and Risk. Each strategy is hosted, self-evolving, configurable through 30+ tunable para...
- **[zubair-trabzada/ai-trading-claude](https://github.com/zubair-trabzada/ai-trading-claude)** — 291 stars — Python: AI trading research engine for Claude Code. Analyze stocks (technical, fundamental, sentiment, risk, thesis), options strategies, sector rotation, portfolio analysis, and PDF reports. 16 skills, 5 parallel agents. Res...
- **[okx/agent-skills](https://github.com/okx/agent-skills)** — 197 stars — Shell: Plug-and-play AI agent skills for OKX — letting any LLM agent trade, manage portfolios, query live market data, and run grid/DCA bots through a single okx CLI, no API wiring required.
- **[studiogangster/sensibull-realtime-options-api-ingestor](https://github.com/studiogangster/sensibull-realtime-options-api-ingestor)** — 76 stars — JavaScript: Sensibull Realtime Option-chain Data: To empower your trading strategies with real-time options data for Algo-Trading and Scalping
- **[woung717/nautilus-trader-cython-stubs](https://github.com/woung717/nautilus-trader-cython-stubs)** — 56 stars — Python: Type stubs for NautilusTrader
- **[HimanshuJ16/Algo-Trading-Skills](https://github.com/HimanshuJ16/Algo-Trading-Skills)** — 50 stars — Python: 500+ algorithmic-trading skills for AI coding agents in the agentskills.io format. Every skill ships a working Python reference implementation and its own tests - 20,291 in CI. Broker APIs, backtesting, risk, executio...
- **[stefansimik/nautilus_trader_examples](https://github.com/stefansimik/nautilus_trader_examples)** — 50 stars — Python — archived (read-only): 
- **[Sergey-1221/nautilus_trader_streamlit](https://github.com/Sergey-1221/nautilus_trader_streamlit)** — 48 stars — Python: 
- **[Black101081/nautilus-trader-admin](https://github.com/Black101081/nautilus-trader-admin)** — 40 stars — TypeScript: NautilusTrader Admin Interface - Complete trading platform management system with 19 admin pages, 4 database backends, and 64 feature management
- **[weixshaw/qlib-docs-cn](https://github.com/weixshaw/qlib-docs-cn)** — 36 stars — HTML: Complete Chinese-language documentation for Microsoft's Qlib quantitative investment platform
- **[DevRico003/mt5-rest-api](https://github.com/DevRico003/mt5-rest-api)** — 14 stars — MQL4: Turn MetaTrader 5 into a REST API server for algorithmic trading - supports historical candles, account data, and trade execution via HTTP.
- **[BobbyAxerol/quantbt](https://github.com/BobbyAxerol/quantbt)** — 11 stars — Python: Event-Drivent, Vectorised-Backtest, Explicit-Order, WalkForward-OP SDK with Numba-compiled simulation kernels and Rust, nautilus-trader endpoint support validation and comparison.
- **[ChadTimothy/qlib-crypto-trading](https://github.com/ChadTimothy/qlib-crypto-trading)** — 11 stars — Python: Cryptocurrency quantitative trading platform built on Microsoft Qlib with real-time monitoring, backtesting, and investment KPI tracking

- **[facioquo/stock-indicators-dotnet](https://github.com/facioquo/stock-indicators-dotnet)** — 1,249 stars — C#: Stock Indicators for .NET is a C# NuGet package that transforms raw equity, commodity, forex, or cryptocurrency financial market price quotes into technical indicators and trading insights. You'll need this essential...

- **[ErikThiart/ai-stock-dashboard](https://github.com/ErikThiart/ai-stock-dashboard)** — 105 stars — Python: Professional AI-powered stock market dashboard with real-time technical analysis, machine learning price predictions, and intelligent market insights. Built with Python, Streamlit, and scikit-learn.

- **[tuhinmallick/InsiderTrader](https://github.com/tuhinmallick/InsiderTrader)** — 52 stars — Jupyter Notebook: This is a Python package for detecting insider trading activity in stock market data. It uses various machine learning techniques to analyze stock prices and volumes, as well as news articles and other publicly availa...

- **[zmcx16/Norn-StockScreener](https://github.com/zmcx16/Norn-StockScreener)** — 85 stars — JavaScript: Norn-StockScreener is a powerful U.S. stock screening tool that helps investors filter stocks based on key fundamentals, expert strategies, and advanced analytics. It features earnings manipulation detection, sector a...

- **[Options-Visualizer/Options-Visualizer.github.io](https://github.com/Options-Visualizer/Options-Visualizer.github.io)** — 66 stars — JavaScript: Visualize profit/loss for various options strategies.

- **[fabston/TradingView-Webhook-Bot](https://github.com/fabston/TradingView-Webhook-Bot)** — 1,888 stars — Python: Send TradingView alerts to Telegram, Discord, Slack, Twitter and Email.

## Curated lists for further discovery

- [wilsonfreitas/awesome-quant](https://github.com/wilsonfreitas/awesome-quant) — curated list of quant/trading libraries and resources
- [paperswithbacktest/awesome-systematic-trading](https://github.com/paperswithbacktest/awesome-systematic-trading) — curated list for systematic trading across crypto, stocks, futures, options
- [wangzhe3224/awesome-systematic-trading](https://github.com/wangzhe3224/awesome-systematic-trading) — another widely-used systematic trading list
- [botcrypto-io/awesome-crypto-trading-bots](https://github.com/botcrypto-io/awesome-crypto-trading-bots) — crypto-bot-specific list
- [cbailes/awesome-deep-trading](https://github.com/cbailes/awesome-deep-trading) — ML-focused trading resources

## Caveats

- Not exhaustive: results are star-ranked and capped per query; many small but genuine strategy repos exist beyond this list.
- Inclusion reflects the repository's own description and is not a recommendation or verification; backtested performance claims in these repos are not independently checked.
- Star counts change constantly; treat them as a rough popularity signal as of October 28, 2026.
