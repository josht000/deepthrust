
@echo off

cargo build --release
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s001_rsi2_connors --variant rsi5,exit70,gate21,pyramid3,stop3atr,rsi5_exit70,rsi5_gate21,rsi5_pyramid3,rsi5_stop3atr,rsi3_10,rsi4_10,rsi2_15,rsi2_20,rsi3_5,rsi3_15,gate_sma100,gate_sma150,gate_sma250,gate_sma300,exit_sma3,exit_sma7,exit_sma10,rsi_exit60,rsi_exit80,stop2atr,stop4atr,pyramid2,pyramid_add10,rsi5_gate21_exit70,stocks_mega,stocks_large,stocks_small
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s003_rsi4_2575 --variant exit75,exit40,exit45,exit50,exit60,exit65,exit70,exit80,rsi15,rsi20,rsi30,rsi35,rsi2_10,rsi3_15,rsi5_25,rsi6_25,gate_sma100,gate_sma150,gate_sma250,gate_sma300,gate21,stop2atr,stop3atr,stop4atr,stop5atr,exit_sma3,exit_sma10,stocks_mega,stocks_large,stocks_small,pyramid2,pyramid_add10
if errorlevel 1 exit /b %errorlevel%

target\release\deepthrust.exe --strategy s006_double_7s --variant lb5,lb10,spy_only,stop3atr,lb5_spy,lb10_stop,lb2,lb3,lb4,lb6,lb8,lb9,lb12,lb15,lb20,sma50,sma100,sma150,sma250,sma300,stop2atr,stop4atr,stop5atr,stop6atr,broad_indices,sector_etfs,stocks_mega,stocks_large,stocks_small,lb5_sma100,lb10_sma250,lb15_stop3
if errorlevel 1 exit /b %errorlevel%