@echo off
setlocal EnableDelayedExpansion

echo ===================================
echo  fix/wit installer for Windows CMD
echo ===================================
echo.

REM Set variables
set "REPO=animeshkundu/fix"
set "INSTALL_DIR=%LOCALAPPDATA%\fix"
set "MODEL_DIR=%APPDATA%\fix"

REM Parse arguments - default to wit as primary
set "PRIMARY=wit"
set "SECONDARY=fix"
if "%~1"=="fix" (
    set "PRIMARY=fix"
    set "SECONDARY=wit"
)

echo Installing %PRIMARY% as primary, %SECONDARY% as secondary
echo.

REM Create directories
echo Creating directories...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%MODEL_DIR%" mkdir "%MODEL_DIR%"

REM Detect architecture
echo Detecting architecture...
if "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    set "ARCH=x86_64"
) else if "%PROCESSOR_ARCHITECTURE%"=="x86" (
    if defined PROCESSOR_ARCHITEW6432 (
        set "ARCH=x86_64"
    ) else (
        set "ARCH=i686"
    )
) else (
    set "ARCH=x86_64"
)
set "TARGET=%ARCH%-pc-windows-msvc"
echo Detected: Windows %ARCH%
echo.

REM Check for curl
where curl >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: curl is required but not found.
    echo Please install curl or use PowerShell installer instead:
    echo   iwr -useb https://animeshkundu.github.io/fix/install.ps1 ^| iex
    exit /b 1
)

REM Get latest release version
echo Fetching latest release...
for /f "tokens=2 delims=:" %%a in ('curl -s "https://api.github.com/repos/%REPO%/releases/latest" ^| findstr /c:"\"tag_name\""') do (
    set "VERSION=%%a"
)
REM Clean up version string (remove quotes, spaces, commas)
set "VERSION=%VERSION: =%"
set "VERSION=%VERSION:"=%"
set "VERSION=%VERSION:,=%"
echo Latest version: %VERSION%
echo.

REM Download primary binary
echo Downloading %PRIMARY% binary...
set "DOWNLOAD_URL=https://github.com/%REPO%/releases/download/%VERSION%/%PRIMARY%-%TARGET%.zip"
curl -fSL "%DOWNLOAD_URL%" -o "%TEMP%\%PRIMARY%.zip"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to download %PRIMARY% binary
    echo URL: %DOWNLOAD_URL%
    exit /b 1
)

REM Extract primary binary
echo Extracting %PRIMARY%...
tar -xf "%TEMP%\%PRIMARY%.zip" -C "%INSTALL_DIR%" 2>nul
if %ERRORLEVEL% neq 0 (
    REM Fall back to PowerShell for extraction if tar fails
    powershell -Command "Expand-Archive -Path '%TEMP%\%PRIMARY%.zip' -DestinationPath '%INSTALL_DIR%' -Force"
)
del "%TEMP%\%PRIMARY%.zip" 2>nul
echo %PRIMARY% installed to %INSTALL_DIR%
echo.

REM Download secondary binary
echo Downloading %SECONDARY% binary...
set "DOWNLOAD_URL=https://github.com/%REPO%/releases/download/%VERSION%/%SECONDARY%-%TARGET%.zip"
curl -fSL "%DOWNLOAD_URL%" -o "%TEMP%\%SECONDARY%.zip"
if %ERRORLEVEL% neq 0 (
    echo WARNING: Failed to download %SECONDARY% binary
    echo URL: %DOWNLOAD_URL%
    echo Continuing with primary binary only...
    goto :skip_secondary
)

REM Extract secondary binary
echo Extracting %SECONDARY%...
tar -xf "%TEMP%\%SECONDARY%.zip" -C "%INSTALL_DIR%" 2>nul
if %ERRORLEVEL% neq 0 (
    powershell -Command "Expand-Archive -Path '%TEMP%\%SECONDARY%.zip' -DestinationPath '%INSTALL_DIR%' -Force"
)
del "%TEMP%\%SECONDARY%.zip" 2>nul
echo %SECONDARY% installed to %INSTALL_DIR%
:skip_secondary
echo.

REM Check if install dir is in PATH
echo %PATH% | findstr /i /c:"%INSTALL_DIR%" >nul
if %ERRORLEVEL% neq 0 (
    echo Adding %INSTALL_DIR% to PATH...
    setx PATH "%PATH%;%INSTALL_DIR%" >nul 2>&1
    if %ERRORLEVEL% neq 0 (
        echo WARNING: Could not add to PATH automatically.
        echo Please add %INSTALL_DIR% to your PATH manually.
    ) else (
        echo Added to PATH successfully.
    )
) else (
    echo %INSTALL_DIR% is already in PATH
)
echo.

REM Download model for primary binary
if "%PRIMARY%"=="wit" (
    set "MODEL_NAME=qwen3-wit-1.7B.gguf"
    set "MODEL_SIZE=~1GB"
) else (
    set "MODEL_NAME=qwen3-correct-0.6B.gguf"
    set "MODEL_SIZE=~378MB"
)

if exist "%MODEL_DIR%\%MODEL_NAME%" (
    echo Model already exists at %MODEL_DIR%\%MODEL_NAME%
) else (
    echo.
    echo Downloading %PRIMARY% model (%MODEL_SIZE%)...
    echo This may take a while depending on your connection speed.
    curl -fSL --progress-bar "https://huggingface.co/animeshkundu/cmd-correct/resolve/main/%MODEL_NAME%" -o "%MODEL_DIR%\%MODEL_NAME%"
    if %ERRORLEVEL% neq 0 (
        echo WARNING: Model download failed.
        echo You can retry later with: %PRIMARY% --update
    ) else (
        echo Model downloaded to %MODEL_DIR%\%MODEL_NAME%
    )
)
echo.

REM Test installation
echo Testing installation...
"%INSTALL_DIR%\%PRIMARY%.exe" --help >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo PASS: %PRIMARY% binary is working
) else (
    echo WARNING: %PRIMARY% binary test failed
)

echo.
echo ===================================
echo  Installation complete!
echo ===================================
echo.
echo Installed binaries:
echo   - %PRIMARY% (primary)
if exist "%INSTALL_DIR%\%SECONDARY%.exe" echo   - %SECONDARY%
echo.
echo Binary location: %INSTALL_DIR%
echo Model location:  %MODEL_DIR%
echo.
echo IMPORTANT: Restart your terminal for PATH changes to take effect.
echo.
echo Run '%PRIMARY% --help' to get started.
echo.

endlocal
