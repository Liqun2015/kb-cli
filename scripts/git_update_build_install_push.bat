@echo off
setlocal enabledelayedexpansion

REM ============================================================
REM  git_update_and_push.bat
REM
REM  Purpose:
REM    Build kb.exe, install kb, commit the current version update,
REM    and push it to GitHub.
REM
REM  Recommended location:
REM    Put this file in the repository root, or in the scripts folder.
REM
REM  Usage:
REM    git_update_and_push.bat v0.1.4
REM    git_update_and_push.bat v0.1.4 "Update kb-cli to v0.1.4"
REM
REM  Behavior:
REM    1. Find the Git repository root
REM    2. Show current branch and changed files
REM    3. Run cargo check if Cargo.toml exists
REM    4. Run cargo build --release to generate target\release\kb.exe
REM    5. Run cargo install --path . --force to install kb.exe
REM    6. Verify kb command is runnable
REM    7. Ask for confirmation
REM    8. git add .
REM    9. git commit
REM   10. git push
REM   11. Optionally create and push a Git tag
REM ============================================================

if "%~1"=="" (
    echo.
    set /p VERSION=Enter version, for example v0.1.4: 
) else (
    set "VERSION=%~1"
)

if "%VERSION%"=="" (
    echo [ERROR] Version is empty.
    pause
    exit /b 1
)

if "%~2"=="" (
    set "COMMIT_MSG=Update kb-cli to %VERSION%"
) else (
    set "COMMIT_MSG=%~2"
)

set "TAG_NAME=%VERSION%"
if /I not "%TAG_NAME:~0,1%"=="v" set "TAG_NAME=v%TAG_NAME%"

set "SCRIPT_DIR=%~dp0"

REM ------------------------------------------------------------
REM 1. Find repository root
REM ------------------------------------------------------------

cd /d "%SCRIPT_DIR%"
git rev-parse --is-inside-work-tree >nul 2>nul

if errorlevel 1 (
    cd /d "%SCRIPT_DIR%.."
    git rev-parse --is-inside-work-tree >nul 2>nul
)

if errorlevel 1 (
    echo.
    echo [ERROR] This script is not inside a Git repository.
    echo Please put it in the repository root or scripts folder.
    pause
    exit /b 1
)

for /f "delims=" %%R in ('git rev-parse --show-toplevel') do set "REPO_ROOT=%%R"

cd /d "%REPO_ROOT%"

for /f "delims=" %%B in ('git rev-parse --abbrev-ref HEAD') do set "BRANCH=%%B"

echo.
echo ============================================================
echo  Git version update, build, install, and push
echo ============================================================
echo  repository : %REPO_ROOT%
echo  branch     : %BRANCH%
echo  version    : %VERSION%
echo  tag        : %TAG_NAME%
echo  commit msg : %COMMIT_MSG%
echo ============================================================
echo.

REM ------------------------------------------------------------
REM 2. Basic branch warning
REM ------------------------------------------------------------

if /I not "%BRANCH%"=="master" (
    if /I not "%BRANCH%"=="main" (
        echo [WARNING] You are not on master/main.
        echo           Current branch: %BRANCH%
        echo.
        set /p CONTINUE_BRANCH=Continue on this branch? Type YES to continue: 
        if /I not "!CONTINUE_BRANCH!"=="YES" (
            echo [ABORTED] No changes were committed.
            pause
            exit /b 1
        )
    )
)

REM ------------------------------------------------------------
REM 3. Show status and change summary
REM ------------------------------------------------------------

echo.
echo [STEP 1] Current Git status:
git status --short

set "STATUS_FILE=%TEMP%\git_status_%RANDOM%_%RANDOM%.txt"
git status --porcelain > "%STATUS_FILE%"

for %%A in ("%STATUS_FILE%") do set "STATUS_SIZE=%%~zA"

if "%STATUS_SIZE%"=="0" (
    del "%STATUS_FILE%" >nul 2>nul
    echo.
    echo [INFO] Working tree is clean. Nothing to commit.
    echo        Build and install can still be tested manually:
    echo        cargo build --release
    echo        cargo install --path . --force
    pause
    exit /b 0
)

del "%STATUS_FILE%" >nul 2>nul

echo.
echo [STEP 2] Change summary:
git diff --stat

REM ------------------------------------------------------------
REM 4. Cargo check, release build, and install
REM ------------------------------------------------------------

if exist "Cargo.toml" (
    echo.
    echo [STEP 3] Running cargo check...
    cargo check
    if errorlevel 1 (
        echo.
        echo [ERROR] cargo check failed.
        echo Fix the Rust errors first. No commit was created.
        pause
        exit /b 1
    )
    echo [OK] cargo check passed.

    echo.
    echo [STEP 4] Running cargo build --release...
    cargo build --release
    if errorlevel 1 (
        echo.
        echo [ERROR] cargo build --release failed.
        echo Fix the Rust build errors first. No commit was created.
        pause
        exit /b 1
    )

    if exist "target\release\kb.exe" (
        echo [OK] Generated release executable:
        echo      %REPO_ROOT%\target\release\kb.exe
    ) else (
        echo.
        echo [ERROR] cargo build --release finished, but target\release\kb.exe was not found.
        echo Check Cargo.toml [[bin]] name. It should usually be:
        echo   name = "kb"
        pause
        exit /b 1
    )

    echo.
    echo [STEP 5] Installing kb.exe with cargo install --path . --force...
    cargo install --path . --force
    if errorlevel 1 (
        echo.
        echo [ERROR] cargo install --path . --force failed.
        echo No commit was created.
        pause
        exit /b 1
    )

    echo.
    echo [STEP 6] Verifying installed kb command...
    where kb
    if errorlevel 1 (
        echo.
        echo [ERROR] kb was installed, but the command was not found in PATH.
        echo Please check whether %%USERPROFILE%%\.cargo\bin is in your PATH.
        pause
        exit /b 1
    )

    kb --help >nul 2>nul
    if errorlevel 1 (
        echo.
        echo [ERROR] kb command exists but cannot run correctly.
        pause
        exit /b 1
    )
    echo [OK] kb command is installed and runnable.
) else (
    echo.
    echo [STEP 3] Cargo.toml not found. Skipping cargo check/build/install.
)

REM ------------------------------------------------------------
REM 5. Confirm commit and push
REM ------------------------------------------------------------

echo.
echo [STEP 7] Ready to commit and push.
echo.
echo Repository:
echo   %REPO_ROOT%
echo Branch:
echo   %BRANCH%
echo Commit message:
echo   %COMMIT_MSG%
echo.

set /p CONFIRM=Type YES to run git add, commit, and push: 

if /I not "%CONFIRM%"=="YES" (
    echo [ABORTED] No changes were committed.
    pause
    exit /b 1
)

REM ------------------------------------------------------------
REM 6. Add and commit
REM ------------------------------------------------------------

echo.
echo [STEP 8] git add .
git add .
if errorlevel 1 (
    echo [ERROR] git add failed.
    pause
    exit /b 1
)

echo.
echo [STEP 9] Staged changes:
git status --short

echo.
echo [STEP 10] git commit
git commit -m "%COMMIT_MSG%"
if errorlevel 1 (
    echo.
    echo [ERROR] git commit failed.
    echo This may happen if there are no staged changes.
    pause
    exit /b 1
)

REM ------------------------------------------------------------
REM 7. Push
REM ------------------------------------------------------------

echo.
echo [STEP 11] git push
git push
if errorlevel 1 (
    echo.
    echo [WARNING] Plain git push failed.
    echo Trying: git push -u origin %BRANCH%
    git push -u origin "%BRANCH%"
    if errorlevel 1 (
        echo.
        echo [ERROR] git push failed.
        pause
        exit /b 1
    )
)

REM ------------------------------------------------------------
REM 8. Optional tag
REM ------------------------------------------------------------

echo.
set /p CREATE_TAG=Create and push Git tag "%TAG_NAME%"? Type Y to create, or press Enter to skip: 

if /I "%CREATE_TAG%"=="Y" (
    git rev-parse "%TAG_NAME%" >nul 2>nul
    if not errorlevel 1 (
        echo [INFO] Tag already exists locally: %TAG_NAME%
    ) else (
        echo.
        echo [STEP 12] Creating tag: %TAG_NAME%
        git tag "%TAG_NAME%"
        if errorlevel 1 (
            echo [ERROR] Failed to create tag.
            pause
            exit /b 1
        )
    )

    echo.
    echo [STEP 13] Pushing tag: %TAG_NAME%
    git push origin "%TAG_NAME%"
    if errorlevel 1 (
        echo [ERROR] Failed to push tag.
        pause
        exit /b 1
    )
) else (
    echo [INFO] Tag creation skipped.
)

REM ------------------------------------------------------------
REM 9. Done
REM ------------------------------------------------------------

echo.
echo ============================================================
echo  Done.
echo ============================================================
echo  Built executable:
echo    %REPO_ROOT%\target\release\kb.exe
echo.
echo  Installed command:
where kb
echo.
echo  Pushed branch:
echo    %BRANCH%
echo.
echo  Commit message:
echo    %COMMIT_MSG%
echo ============================================================
echo.

pause
exit /b 0
