@echo off
setlocal ENABLEEXTENSIONS

@REM Get the installed configuration
set KEY_NAME=HKLM\SYSTEM\CurrentControlSet\Services\npcap\Parameters
for /F "usebackq tokens=1,2*" %%A IN (`reg query "%KEY_NAME%" /v "Dot11Support" 2^>nul ^| find "Dot11Support"`) do (
	set Dot11Support=%%C
)
echo Dot11Support = %Dot11Support%
for /F "usebackq tokens=1,2*" %%A IN (`reg query "%KEY_NAME%" /v "LoopbackAdapter" 2^>nul ^| find "LoopbackAdapter"`) do (
	set LoopbackAdapter=%%C
)
echo LoopbackAdapter = %LoopbackAdapter%

@REM Make sure we can find where Npcap is installed
set KEY_NAME=HKLM\Software\WOW6432Node\Npcap
for /F "usebackq tokens=1,2*" %%A IN (`reg query "%KEY_NAME%" /ve 2^>nul ^| find "REG_SZ"`) do (
	set NPCAP_DIR=%%C
)
if defined NPCAP_DIR (goto DO_::OVE)
set KEY_NAME=HKLM\Software\Npcap
for /F "usebackq tokens=1,2*" %%A IN (`reg query "%KEY_NAME%" /ve 2^>nul ^| find "REG_SZ"`) do (
	set NPCAP_DIR=%%C
)
if defined NPCAP_DIR (goto DO_::OVE) else (goto ABORT)

:DO_::OVE
echo NPCAP_DIR = "%NPCAP_DIR%"
@REM Stop the services and set their start types properly
net stop npcap
if %Dot11Support% == 0x1 (
	net stop npcap_wifi
	@REM *_wifi service is disabled at install
	sc.exe config npcap_wifi start= disabled
)
"%NPCAP_DIR%\Uninstall.exe"

@REM Done!
goto EOF

:ABORT
echo "Unable to find or fix your installation"
exit /b 1

:EOF
exit /b 0