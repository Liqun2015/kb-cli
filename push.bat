@echo off
chcp 65001 >nul

if "%~1"=="" (
    echo Usage: push.bat "commit message"
    echo Example: push.bat "v0.4.7 cross-platform quickstart"
    exit /b 1
)

echo.
echo ===== Current status =====
git status

echo.
echo ===== Unstaged diff: changes not yet added =====
git diff

echo.
echo 请先检查上面的修改内容。
echo 如果确认无误，按任意键继续执行 git add .
pause >nul

git add .

echo.
echo ===== Staged diff: changes ready to commit =====
git diff --cached

echo.
echo 请再次检查即将提交的内容。
echo 如果确认无误，按任意键继续执行 git commit 和 git push
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