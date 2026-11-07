@echo off
setlocal
targetelease\deepthrust.exe --strategy g077_cta_binary --variant f4s12,f8s24,f12s36,f16s48,f20s60,f24s72,f32s96,f32s128,f40s120,f48s144,f64s192,f96s288,cs10,cs25,cs75,cs100,cs150,cs200,warm50,warm150,warm200,warm300,weekly,monthly,f16s48_cs10,f20s60_cs10,f64s192_cs10,f16s48_weekly,f64s192_weekly,f8s24_cs15,f48s144_cs025,f32s128_cs10,f40s120_warm200,f16s48_warm150,f24s72_cs075
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g078_cta_volscaled --variant f8s24,f16s48,f20s60,f24s72,f32s96,f32s128,f48s144,f64s192,vol16,vol32,vol48,vol64,vol96,vol128,cs500,cs2000,cs3000,cs5000,weekly,monthly,warm200,warm400,f16s48_vol64,f64s192_vol64,f20s60_cs002,f48s144_cs002,f32s96_weekly,vol64_cs002,vol16_cs002,vol96_cs002,f16s48_cs002,f64s192_cs002,f24s72_vol48,f40s120_vol48
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g079_cta_robust --variant clip20,clip30,clip42,clip50,clip60,vol16,vol32,vol64,vol96,f8s24,f16s48,f20s60,f32s96,f32s128,f48s144,f64s192,cs500,cs2000,cs3000,weekly,monthly,clip42_f16s48,clip42_f64s192,clip42_vol64,clip42_cs002,clip3_vol64,clip5_vol64,clip3_cs002,clip42_weekly,clip42_f20s60,clip42_f32s128,warm200,warm400
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g080_cta_riskparity --variant clip20,clip30,clip42,clip50,vol16,vol32,vol64,vol96,f8s24,f16s48,f20s60,f32s96,f32s128,f48s144,f64s192,cs25,cs50,cs100,cs200,weekly,monthly,clip3_f16s48,clip3_f64s192,clip3_vol64,clip3_cs01,clip5_vol64,clip42_cs01,clip42_weekly,clip42_f20s60,clip5_cs01,clip3_weekly,warm200,warm400
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g081_cta_dcc --variant corr100,corr150,corr200,corr250,corr300,corr400,sh0,sh25,sh50,sh75,sh100,clip30,clip42,clip50,cs50,cs100,cs200,f16s48,f20s60,f32s96,f32s128,f48s144,f64s192,weekly,corr100_sh25,corr300_sh75,corr150_sh50,corr200_sh0,corr200_sh100,corr100_cs02,corr200_f16s48,corr200_f64s192,corr150_clip3,corr250_sh25,corr100_weekly,warm200,warm400
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g085_local_extreme --variant lb20,lb40,lb60,lb63,lb80,lb100,lb126,lb189,lb252,lb315,tv15,tv25,tv40,tv60,tv80,cap50,cap75,cap95,cap150,cap200,vol20,vol40,vol60,vol120,weekly,dom1,dom15,lb40_tv25,lb126_tv25,lb252_tv25,lb126_tv60,lb63_weekly,lb126_weekly,lb252_weekly,lb40_cap50,lb126_cap50,lb63_vol20,lb126_vol120
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g095_ssa_trend --variant eig15,eig20,eig30,eig40,eig50,m60,m90,m120,m180,m252,step10,step21,step42,step63,tv5,tv10,tv15,tv20,ew,overq,indq,overq_thr30,indq_thr30,overq_thr70,indq_thr70,nolev,m60_eig20,m120_eig40,m180_eig30,m120_step21,ew_m120,ew_step42,indq_ew,overq_ew,eig30_tv15,m252_step21
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g106_xs_com_mom --variant mom63,mom126,mom189,mom252,mom315,mom378,legs1,legs2,legs3,weekly,dom1,dom15,mom126_legs1,mom126_legs3,mom252_legs1,mom252_legs3,mom189_legs1,mom189_legs3,mom63_legs1,mom63_legs3,mom315_legs1,mom315_legs3,mom126_weekly,mom252_weekly,mom189_weekly,mom63_weekly,mom378_legs1,mom378_legs3,legs1_weekly,legs3_weekly,mom126_dom1,mom252_dom1,mom189_dom15,mom189_dom1,mom126_dom15
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g109_xs_skew --variant sk63,sk126,sk189,sk252,sk315,sk378,legs1,legs2,legs3,weekly,dom1,dom15,sk126_legs1,sk126_legs3,sk252_legs1,sk252_legs3,sk189_legs1,sk189_legs3,sk63_legs1,sk63_legs3,sk126_weekly,sk252_weekly,sk189_weekly,sk63_weekly,sk315_legs1,sk315_legs3,legs1_weekly,legs3_weekly,sk126_dom1,sk252_dom1,sk189_dom15,sk189_dom1,sk126_dom15
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g063_tsy_spread --variant z10,z15,z20,z25,z30,t2,t3,t4,t6,t8,up4,up6,up8,up12,dn2,dn4,dn6,dn8,zw40,zw60,zw120,zw180,w05,w10,w15,w20,w25,hold21,hold42,hold63,hold126,units1,units8,z15_t3,z2_t3,z25_t6,z15_up6,z2_dn6,z15_hold42,zw40_z15,w15_z15,w2_z15
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g010_crypto_blend --variant w5,w10,w15,w20,w30,w40,w60,w90,w10_ls,w15_ls,w20_ls,w30_ls,w40_ls,w60_ls,w90_ls,ls,weekly,w10_weekly,w20_weekly,w30_weekly,w60_weekly,w10_ls_weekly,w20_ls_weekly,w30_ls_weekly,w60_ls_weekly,w5_weekly,w15_weekly,w40_weekly,w90_weekly
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g114_swing_scan --variant near2,near3,near5,near10,near15,sma20,sma50,sma100,sma150,sma200,lb126,lb189,lb252,lb315,rr10,rr15,rr20,rr30,hold5,hold10,hold20,hold40,hold60,con5,con10,con15,con20,rsi40,rsi50,rsi60,con10_w20,con15_w7,near03_rr15,near10_sma20,lb126_sma20,con10_rr15,rsi50_rr15,u150,u500,etf_uni
if errorlevel 1 exit /b %errorlevel%
targetelease\deepthrust.exe --strategy g139_vix_ts_pair --variant thr95,thr100,thr105,thr110,z05,z10,z15,z20,zw40,zw60,zw120,zw180,ezm05,ez00,ez05,hold21,hold42,hold63,hold126,w05,w10,w15,w20,thr105_z05,thr105_z15,thr100_z05,thr100_z15,thr110_z10,zw40_z05,zw120_z10,hold21_z05,w05_z05,w15_z05,thr095_hold126,thr105_ezm05,thr100_zw40
if errorlevel 1 exit /b %errorlevel%
echo DONE
