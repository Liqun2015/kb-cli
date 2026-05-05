@echo off
setlocal enabledelayedexpansion

REM ============================================================
REM  build_llm_wiki.bat
REM
REM  Windows quick-start wrapper for the cross-platform command:
REM    kb --kb-path <target> bootstrap --copy
REM
REM  Basic usage:
REM    scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum"
REM
REM  With explicit kb-cli source directory:
REM    scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" "D:\github\LLM-wiki\kb-cli"
REM
REM  Default behavior:
REM    - Safe COPY mode, not move mode.
REM    - Only root-level source files are organized.
REM    - Use --move if you intentionally want to move files into raw\.
REM ============================================================

if "%~1"=="" goto :help
if /I "%~1"=="--help" goto :help
if /I "%~1"=="-h" goto :help
if /I "%~1"=="/?" goto :help

set "KB_ROOT=%~1"
shift

set "KB_CLI_DIR="
set "MODE=copy"
set "BOOTSTRAP_EXTRA_ARGS="
set "NO_INSTALL=0"
set "NO_PAUSE=0"

REM Optional second positional argument: kb-cli source directory.
if not "%~1"=="" (
    set "ARG=%~1"
    if not "!ARG:~0,2!"=="--" (
        set "KB_CLI_DIR=%~1"
        shift
    )
)

:parse_args
if "%~1"=="" goto :args_done
if /I "%~1"=="--copy" (
    set "MODE=copy"
) else if /I "%~1"=="--move" (
    set "MODE=move"
) else if /I "%~1"=="--recursive" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --recursive"
) else if /I "%~1"=="--dry-run" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --dry-run"
) else if /I "%~1"=="--force-init" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --force-init"
) else if /I "%~1"=="--force-metadata" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --force-metadata"
) else if /I "%~1"=="--skip-metadata" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --skip-metadata"
) else if /I "%~1"=="--skip-build" (
    set "BOOTSTRAP_EXTRA_ARGS=!BOOTSTRAP_EXTRA_ARGS! --skip-build"
) else if /I "%~1"=="--no-install" (
    set "NO_INSTALL=1"
) else if /I "%~1"=="--no-pause" (
    set "NO_PAUSE=1"
) else if /I "%~1"=="--help" (
    goto :help
) else (
    echo [ERROR] Unknown argument: %~1
    echo.
    goto :help
)
shift
goto :parse_args

:args_done

set "BOOTSTRAP_ARGS=--%MODE%%BOOTSTRAP_EXTRA_ARGS%"

if "%KB_CLI_DIR%"=="" (
    set "KB_CLI_DIR=%~dp0.."
    if not exist "%KB_CLI_DIR%\Cargo.toml" (
        set "KB_CLI_DIR=%CD%"
    )
)

echo.
echo ============================================================
echo  kb-cli LLM Wiki quick builder
echo ============================================================
echo  target wiki root : %KB_ROOT%
echo  kb-cli source    : %KB_CLI_DIR%
echo  bootstrap args   : %BOOTSTRAP_ARGS%
echo ============================================================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] cargo was not found.
    echo Please install Rust first, then re-run this script.
    goto :fail
)

where kb >nul 2>nul
if errorlevel 1 (
    echo [INFO] kb command not found.
    if "%NO_INSTALL%"=="1" (
        echo [ERROR] --no-install was set, so this script will not run cargo install.
        echo Please install manually first:
        echo   cd /d "%KB_CLI_DIR%"
        echo   cargo install --path . --force
        goto :fail
    )

    if exist "%KB_CLI_DIR%\Cargo.toml" (
        echo [INFO] Installing kb-cli from:
        echo        %KB_CLI_DIR%
        pushd "%KB_CLI_DIR%"
        cargo install --path . --force
        if errorlevel 1 (
            popd
            echo [ERROR] Failed to install kb-cli.
            goto :fail
        )
        popd
    ) else (
        echo [ERROR] kb command is not installed, and Cargo.toml was not found in:
        echo         %KB_CLI_DIR%
        echo.
        echo Please either install kb first or pass kb-cli source path as the second argument.
        goto :fail
    )
) else (
    echo [OK] kb command found.
)

kb --help >nul 2>nul
if errorlevel 1 (
    echo [ERROR] kb command exists but cannot run correctly.
    goto :fail
)

echo [OK] kb is runnable.
echo.
echo [RUN] kb --kb-path "%KB_ROOT%" bootstrap %BOOTSTRAP_ARGS%
echo.

kb --kb-path "%KB_ROOT%" bootstrap %BOOTSTRAP_ARGS%
if errorlevel 1 (
    echo [ERROR] kb bootstrap failed.
    goto :fail
)

echo.
echo ============================================================
echo  Done.
echo ============================================================
echo  Wiki home:
echo  %KB_ROOT%\wiki\Home.md
echo.
echo  Open this folder with Obsidian:
echo  %KB_ROOT%
echo ============================================================
echo.

if not "%NO_PAUSE%"=="1" pause
exit /b 0

:fail
echo.
echo [FAILED] build_llm_wiki.bat stopped.
echo.
if not "%NO_PAUSE%"=="1" pause
exit /b 1

:help
echo.
echo kb-cli Windows quick-start helper
echo.
echo Usage:
echo   scripts\build_llm_wiki.bat "D:\path\to\KnowledgeBase"
echo.
echo Optional:
echo   scripts\build_llm_wiki.bat "D:\path\to\KnowledgeBase" "D:\path\to\kb-cli"
echo.
echo Flags:
echo   --copy             Copy files into raw\, default and safest.
echo   --move             Move files into raw\.
echo   --recursive        Also collect files from subfolders, excluding raw/wiki/rules/etc.
echo   --dry-run          Show planned actions without copying or moving files.
echo   --force-init       Run kb init --force.
echo   --force-metadata   Re-extract PDF metadata.
echo   --skip-metadata    Skip PDF metadata extraction.
echo   --skip-build       Skip wiki generation.
echo   --no-install       Do not auto-install kb with cargo install.
echo   --no-pause         Do not pause when finished.
echo   --help             Show this help.
echo.
echo Examples:
echo   scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum"
echo   scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --move
echo   scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --recursive --dry-run
echo.
pause
exit /b 1
