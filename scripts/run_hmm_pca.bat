@echo off
setlocal
pushd "%~dp0.."
cargo build --release --bin deepthrust
if errorlevel 1 goto :failure
target\release\deepthrust.exe --strategy hmm_regime --config configs\hmm_ablation.yaml --variant pca_a_full,pca_b2_diag,pca_b3_diag,pca_b4_diag,pca_b5_diag,pca_b6_diag,pca_b2_full,pca_b3_full,pca_b4_full,pca_b5_full,pca_b6_full,pca_c_diag,pca_c_full,pca_d2_diag,pca_d3_diag,pca_d4_diag,pca_d5_diag,pca_d2_full,pca_d3_full,pca_d4_full,pca_d5_full,pca_e_diag,pca_e_full,pca_f2_diag,pca_f3_diag,pca_f4_diag,pca_f5_diag,pca_f6_diag,pca_f2_full,pca_f3_full,pca_f4_full,pca_f5_full,pca_f6_full
if errorlevel 1 goto :failure
C:\data\envs\trading_framework\Scripts\python.exe tools\update_all_trade_metrics.py
if errorlevel 1 goto :failure
C:\data\envs\trading_framework\Scripts\python.exe tools\hmm_pca_results.py
if errorlevel 1 goto :failure
popd
exit /b 0

:failure
set "result=%errorlevel%"
popd
exit /b %result%
