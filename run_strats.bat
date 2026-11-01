
@echo off
setlocal

set "DEBUG_ARGS="
if /I "%~1"=="--debug" set "DEBUG_ARGS=--debug"

cargo build --release
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s001_rsi2_connors --variant rsi5,exit70,gate21,pyramid3,stop3atr,rsi5_exit70,rsi5_gate21,rsi5_pyramid3,rsi5_stop3atr,rsi3_10,rsi4_10,rsi2_15,rsi2_20,rsi3_5,rsi3_15,gate_sma100,gate_sma150,gate_sma250,gate_sma300,exit_sma3,exit_sma7,exit_sma10,rsi_exit60,rsi_exit80,stop2atr,stop4atr,pyramid2,pyramid_add10,rsi5_gate21_exit70,stocks_mega,stocks_large,stocks_small,mixed_mega %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s003_rsi4_2575 --variant exit75,exit40,exit45,exit50,exit60,exit65,exit70,exit80,rsi15,rsi20,rsi30,rsi35,rsi2_10,rsi3_15,rsi5_25,rsi6_25,gate_sma100,gate_sma150,gate_sma250,gate_sma300,gate21,stop2atr,stop3atr,stop4atr,stop5atr,exit_sma3,exit_sma10,stocks_mega,stocks_large,stocks_small,mixed_mega,pyramid2,pyramid_add10 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s006_double_7s --variant lb5,lb10,spy_only,stop3atr,lb5_spy,lb10_stop,lb2,lb3,lb4,lb6,lb8,lb9,lb12,lb15,lb20,sma50,sma100,sma150,sma250,sma300,stop2atr,stop4atr,stop5atr,stop6atr,broad_indices,sector_etfs,stocks_mega,stocks_large,stocks_small,mixed_mega,lb5_sma100,lb10_sma250,lb15_stop3 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s017_night_trading --variant quarterly,monthly_s034,full_day,overnight_intraday,negative_intraday,top10,top30,top40,fdr01,fdr10,t15,t25,t30,window126,window189,window315,quarterly_top10,quarterly_top30,monthly_top10,monthly_top30,fdr01_top10,fdr10_top30,t25_fdr01,t15_fdr10,full_day_top10,full_day_top30,spread_top10,spread_top30,negative_top10,negative_quarterly %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s021_index_overnight --variant sma50,qqq,iwm,cost05,cost2,cost5,cost025,cost10,sma50_cost05,sma50_cost2,sma50_cost5,sma50_cost10,qqq_cost025,qqq_cost05,qqq_cost2,qqq_cost5,qqq_cost10,iwm_cost025,iwm_cost05,iwm_cost2,iwm_cost5,iwm_cost10,qqq_sma50,iwm_sma50,qqq_sma50_cost05,qqq_sma50_cost2,qqq_sma50_cost5,iwm_sma50_cost05,iwm_sma50_cost2,iwm_sma50_cost5 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s032_52w_high_proximity --variant ratio80,hold6_skip1,high126,high189,high315,top05,top15,universe100,universe250,universe1000,high126_top05,high126_top15,high189_top05,high189_top15,high315_top05,high315_top15,high126_n100,high126_n1000,high189_n100,high189_n1000,high315_n100,high315_n1000,top05_n100,top05_n1000,top15_n100,top15_n1000,ratio80_n100,ratio80_n1000,hold6_skip1_high315,hold6_skip1_top15 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s035_low_volatility --variant vol60,vol126,vol504,universe100,universe250,universe1000,decile05,decile15,sector_neutral,low_beta,low_beta_sector,cost5,cost25,vol60_n100,vol60_n1000,vol126_n100,vol126_n1000,vol504_n100,vol504_n1000,n100_decile05,n100_decile15,n1000_decile05,n1000_decile15,vol60_sector,vol126_sector,vol504_sector,low_beta_n100,low_beta_n1000,low_beta_cost5,low_beta_cost25,low_beta_vol60 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s033_52w_breakout --variant entry63_exit21,entry63_exit63,entry63_exit126,entry126_exit21,entry126_exit63,entry126_exit126,entry189_exit21,entry189_exit63,entry189_exit126,entry252_exit21,entry252_exit63,entry315_exit21,entry315_exit63,entry315_exit126,entry63_exit21_top100,entry63_exit63_top100,entry63_exit126_top100,entry126_exit21_top100,entry126_exit63_top100,entry126_exit126_top100,entry189_exit21_top100,entry189_exit63_top100,entry189_exit126_top100,entry252_exit21_top100,entry252_exit63_top100,etf_only,etf_only_fast,etf_only_slow,top250,top1000 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s040_volume_breakout --variant quiet_breakout,limit_close,w10_d005_v125,w10_d005_v150,w10_d005_v200,w10_d010_v125,w10_d010_v150,w10_d010_v200,w10_d020_v125,w10_d020_v150,w10_d020_v200,w20_d005_v125,w20_d005_v150,w20_d005_v200,w20_d010_v125,w20_d010_v200,w20_d020_v125,w20_d020_v150,w20_d020_v200,w40_d005_v125,w40_d005_v150,w40_d005_v200,w40_d010_v125,w40_d010_v150,w40_d010_v200,w40_d020_v125,w40_d020_v150,w40_d020_v200,w30_d010_v150 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s041_mom_12_1 --variant mom126_skip0,mom126_skip10,mom126_skip21,mom126_skip42,mom189_skip0,mom189_skip10,mom189_skip21,mom189_skip42,mom252_skip0,mom252_skip10,mom252_skip42,mom315_skip0,mom315_skip10,mom315_skip21,mom315_skip42,mom378_skip0,mom378_skip10,mom378_skip21,mom378_skip42,top10,top20,top30,top40,top50,top75,top100,top150,top200,decile5,decile15,indneutral,top10_ind %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s042_sector_momentum --variant rankavg,blend13,top1,top2,top5,top7,top9,weekly,rankavg_top1,rankavg_top2,rankavg_top5,rankavg_top7,rankavg_top9,rankavg_weekly,blend13_top1,blend13_top2,blend13_top5,blend13_top7,blend13_top9,blend13_weekly,mom12_top1_weekly,mom12_top2_weekly,mom12_top5_weekly,mom12_top7_weekly,mom12_top9_weekly,rankavg_top1_weekly,rankavg_top2_weekly,rankavg_top5_weekly,rankavg_top7_weekly,rankavg_top9_weekly %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s043_etf_multi_asset_momentum --variant mom63_top1,mom63_top2,mom63_top3,mom63_top5,mom63_top7,mom126_top1,mom126_top2,mom126_top3,mom126_top5,mom126_top7,mom189_top1,mom189_top2,mom189_top3,mom189_top5,mom189_top7,mom252_top1,mom252_top2,mom252_top3,mom252_top7,mom315_top1,mom315_top2,mom315_top3,mom315_top5,mom315_top7,mom378_top1,mom378_top2,mom378_top3,mom378_top5,mom378_top7,mom504_top5 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s044_efficiente5 --variant target10,lookback252,lb63_t5,lb63_t6,lb63_t7,lb63_t8,lb63_t10,lb84_t5,lb84_t6,lb84_t7,lb84_t8,lb84_t10,lb126_t6,lb126_t7,lb126_t8,lb189_t5,lb189_t6,lb189_t7,lb189_t8,lb189_t10,lb252_t5,lb252_t6,lb252_t7,lb252_t8,lb315_t5,lb315_t6,lb315_t7,lb315_t8,lb315_t10 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s045_characteristic_blend --variant paper_design,catalog_simplification,a_n250_v10,a_n250_v15,a_n250_v25,a_n250_v30,a_n500_v10,a_n500_v15,a_n500_v25,a_n500_v30,a_n750_v10,a_n750_v15,a_n750_v25,a_n750_v30,a_n1000_v10,a_n1000_v15,a_n1000_v25,a_n1000_v30,b_n250_v10,b_n250_v15,b_n250_v25,b_n500_v10,b_n500_v15,b_n500_v25,b_n750_v10,b_n750_v15,b_n750_v25,b_n1000_v10,b_n1000_v15,b_n1000_v25 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s046_market_state_industry_rotation --variant analog25,analog100,long_only,k25_f36_ls,k25_f84_ls,k50_f36_ls,k50_f84_ls,k75_f36_ls,k75_f60_ls,k75_f84_ls,k100_f36_ls,k100_f84_ls,k150_f36_ls,k150_f60_ls,k150_f84_ls,k25_f36_lo,k25_f60_lo,k25_f84_lo,k50_f36_lo,k50_f84_lo,k75_f36_lo,k75_f60_lo,k75_f84_lo,k100_f36_lo,k100_f60_lo,k100_f84_lo,k150_f36_lo,k150_f60_lo,k150_f84_lo %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s047_absolute_dual_momentum --variant gem,absolute_tlt,sma10_gate,abs63_bil,abs63_tlt,abs126_bil,abs126_tlt,abs189_bil,abs189_tlt,abs315_bil,abs315_tlt,abs378_bil,gem63,gem126,gem189,gem315,gem378,gem504,gem630,gem756,gem1008,sma126_bil,sma126_tlt,sma168_bil,sma168_tlt,sma210_tlt,sma252_bil,sma252_tlt,sma315_bil,sma315_tlt %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s050_first_pullback --variant breakout42_pullback2_dist1,breakout42_pullback2_dist2,breakout42_pullback2_dist3,breakout42_pullback3_dist1,breakout42_pullback3_dist2,breakout42_pullback3_dist3,breakout42_pullback4_dist1,breakout42_pullback4_dist2,breakout42_pullback4_dist3,breakout63_pullback2_dist1,breakout63_pullback2_dist2,breakout63_pullback2_dist3,breakout63_pullback3_dist1,breakout63_pullback3_dist2,breakout63_pullback3_dist3,breakout63_pullback4_dist1,breakout63_pullback4_dist2,breakout63_pullback4_dist3,breakout126_pullback2_dist1,breakout126_pullback2_dist2,breakout126_pullback2_dist3,breakout126_pullback3_dist1,breakout126_pullback3_dist2,breakout126_pullback3_dist3,breakout126_pullback4_dist1,breakout126_pullback4_dist2,breakout126_pullback4_dist3,avoid_earnings,breakout126_pullback4_dist2_avoid_earnings %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s051_darvas_box --variant u100_h126_close,u100_h126_next,u100_h250_close,u100_h250_next,u100_h500_close,u100_h500_next,u250_h126_close,u250_h126_next,u250_h250_close,u250_h250_next,u250_h500_close,u250_h500_next,u500_h126_close,u500_h126_next,u500_h250_close,u500_h250_next,u500_h500_close,u500_h500_next,u750_h126_close,u750_h126_next,u750_h250_close,u750_h250_next,u750_h500_close,u750_h500_next,u1000_h126_close,u1000_h126_next,u1000_h250_close,u1000_h250_next,u1000_h500_close,u1000_h500_next %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s075_residual_reversal --variant skip1,win3,win10,dec05,zscore,nosma,big5,big8,big10,exit_z05,fixed5,fixed3,stop8,cap5,ew_dd10,ew_vol30,big5_ez05,big8_ez05,big5_dec05,peer_industry,beta40,beta90,min_peers3,peer_top1000,decile15,zscore_both,stop10,target10,fixed10,skip3 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s105_overnight_carry --variant cap2x,daily_resize,target5,target15,window10,window40,cap05,cap15,target25,target75,target20,target25pct,window5,window15,window30,window60,band10,band40,cap05_target5,cap2_target5,cap2_target15,window10_target5,window10_target15,window40_target5,window40_target15,daily_cap2,daily_target5,daily_target15,band10_target5,band40_target15 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%