
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

target\release\deepthrust.exe --strategy s021_index_overnight --variant sma50,qqq,iwm,cost05,cost2,cost5,cost025,cost10,sma50_cost05,sma50_cost2,sma50_cost5,sma50_cost10,qqq_cost025,qqq_cost05,qqq_cost2,qqq_cost5,qqq_cost10,iwm_cost025,iwm_cost05,iwm_cost2,iwm_cost5,iwm_cost10,qqq_sma50,iwm_sma50,qqq_sma50_cost05,qqq_sma50_cost2,qqq_sma50_cost5,iwm_sma50_cost05,iwm_sma50_cost2,iwm_sma50_cost5 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s033_52w_breakout --variant entry63_exit21,entry63_exit63,entry63_exit126,entry126_exit21,entry126_exit63,entry126_exit126,entry189_exit21,entry189_exit63,entry189_exit126,entry252_exit21,entry252_exit63,entry315_exit21,entry315_exit63,entry315_exit126,entry63_exit21_top100,entry63_exit63_top100,entry63_exit126_top100,entry126_exit21_top100,entry126_exit63_top100,entry126_exit126_top100,entry189_exit21_top100,entry189_exit63_top100,entry189_exit126_top100,entry252_exit21_top100,entry252_exit63_top100,etf_only,etf_only_fast,etf_only_slow,top250,top1000 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s041_mom_12_1 --variant mom126_skip0,mom126_skip10,mom126_skip21,mom126_skip42,mom189_skip0,mom189_skip10,mom189_skip21,mom189_skip42,mom252_skip0,mom252_skip10,mom252_skip42,mom315_skip0,mom315_skip10,mom315_skip21,mom315_skip42,mom378_skip0,mom378_skip10,mom378_skip21,mom378_skip42,top10,top20,top30,top40,top50,top75,top100,top150,top200,decile5,decile15 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s042_sector_momentum --variant rankavg,blend13,top1,top2,top5,top7,top9,weekly,rankavg_top1,rankavg_top2,rankavg_top5,rankavg_top7,rankavg_top9,rankavg_weekly,blend13_top1,blend13_top2,blend13_top5,blend13_top7,blend13_top9,blend13_weekly,mom12_top1_weekly,mom12_top2_weekly,mom12_top5_weekly,mom12_top7_weekly,mom12_top9_weekly,rankavg_top1_weekly,rankavg_top2_weekly,rankavg_top5_weekly,rankavg_top7_weekly,rankavg_top9_weekly %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s105_overnight_carry --variant cap2x,daily_resize,target5,target15,window10,window40,cap05,cap15,target25,target75,target20,target25pct,window5,window15,window30,window60,band10,band40,cap05_target5,cap2_target5,cap2_target15,window10_target5,window10_target15,window40_target5,window40_target15,daily_cap2,daily_target5,daily_target15,band10_target5,band40_target15 %DEBUG_ARGS%
if errorlevel 1 exit /b %errorlevel%