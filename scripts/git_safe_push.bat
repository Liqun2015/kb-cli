@echo off
chcp 65001 >nul

if "%~1"=="" (
    echo Usage: scripts\git_safe_push.bat "commit message"
    echo Example: scripts\git_safe_push.bat "v0.4.8 developer workflow helpers"
    exit /b 1
)

echo.
echo ===== Current status =====
git --no-pager status

echo.
echo ===== Unstaged diff: changes not yet added =====
git --no-pager diff

echo.
echo Please review the changes above.
echo Press any key to continue with: git add .
pause >nul

git add .

echo.
echo ===== Staged diff: changes ready to commit =====
git --no-pager diff --cached

echo.
echo Please review the staged changes above.
echo Press any key to continue with commit and push.
pause >nul

git commit -m "%~1"

if errorlevel 1 (
    echo.
    echo git commit failed. Push aborted.
    exit /b 1
)

git push

if errorlevel 1 (
    echo.
    echo git push failed.
    exit /b 1
)

echo.
echo Done.
