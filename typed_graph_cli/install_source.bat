@echo off

SET BinPath="%USERPROFILE%\bin"
set ArbiterPath=%BinPath%\arbiter.cmd

if not exist %BinPath% (
    mkdir %BinPath%
)

cargo build --release

copy /Y ..\target\release\typed_graph.exe %BinPath%\typed_graph.exe

setlocal enabledelayedexpansion
set replaced=%PATH%

set replaced=!replaced:%BinPath%=!

if "%replaced%" == "%PATH%" echo IMPORTANT: Add %BinPath% to the PATH enviromental variable