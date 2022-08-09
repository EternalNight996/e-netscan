@echo off

@REM @REM  get administrator permit
@REM %1 mshta vbscript:CreateObject("Shell.Application").ShellExecute("cmd.exe","/c %~s0 ::","","runas",1)(window.close)&&exit
@REM @REM delay for 1/s
TIMEOUT /T 1

if "%E_NETSCAN_DIR%" == "" (
    SET ERROR_CODE="Couldn't read [ E_NETSCAN_DIR ] of variable " 
    goto ERROR
)

if EXIST %LIB% ( goto FOUND ) else ( goto NOT_FOUND )

:NOT_FOUND
echo "NOT FOUND [LIB]"
set CREATE_DIR="%USERPROFILE%\libs"
@REM set new forever variable
setx LIB "%USERPROFILE%\libs"
if NOT EXIST "%USERPROFILE%\libs" (
    echo "Trying create new dir to %USERPROFILE%\libs"
    MKDIR "%USERPROFILE%\libs"
) else (
    echo "FOUND DIR [%CREATE_DIR%]"
)
IF NOT EXIST "%CREATE_DIR%\Packet.lib" (
    ECHO "Trying copy from %E_NETSCAN_DIR%\static\libs\Packet.lib to %CREATE_DIR%\Packet.lib"
    echo F | xcopy /Y "%E_NETSCAN_DIR%\static\libs\Packet.lib" "%CREATE_DIR%\Packet.lib"
) else (
    echo "FOUND FILE [%LIB%\Packet.lib]"
)
GOTO EOF

:FOUND
@REM if [LIB] not exist then copy to there;
IF NOT EXIST %LIB% (
    echo "Trying create new dir to %LIB%"
    MKDIR %LIB%
) else (
    echo "FOUND DIR [%LIB%]"
)
@REM if not exist then copy to there;
IF NOT EXIST "%LIB%\Packet.lib" (
    ECHO "Trying copy from %E_NETSCAN_DIR%\static\libs\Packet.lib to %LIB%\Packet.lib"
    echo F | xcopy /Y "%E_NETSCAN_DIR%\static\libs\Packet.lib" "%LIB%\Packet.lib"
) else (
    echo "FOUND FILE [%LIB%\Packet.lib]"
)
GOTO EOF

:ERROR
ECHO %ERROR_CODE%
exit 1

:EOF
exit 0