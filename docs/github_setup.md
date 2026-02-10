# 🚀 Pushing to GitHub

Here is how to safely push your Fintech App to GitHub.

## 1. Prepare the Repository
Your `.gitignore` is already set up to exclude sensitive files like `.env` and heavy folders like `target/` and `postgres_data/`.

```bash
# Check what will be committed (optional)
git status
```

## 2. Initialize and Commit
If you haven't already:

```bash
git init
git add .
git commit -m "Initial commit: Fintech App with Wallet Features"
```

## 3. Push to GitHub
1.  Go to [GitHub.com](https://github.com/new) and create a new **empty** repository.
2.  Copy the URL (e.g., `https://github.com/yourusername/my-fintech-app.git`).
3.  Run these commands:

```bash
git branch -M main
git remote add origin <PASTE_YOUR_GITHUB_URL_HERE>
git push -u origin main
```

## ⚠️ Important
- **NEVER** push your `.env` file containing secrets.
- If you need to share config, use a `.env.example` file with dummy values.
