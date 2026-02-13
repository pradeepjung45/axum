# 📧 Gmail SMTP Email Notifications - Setup & Testing Guide

## ✅ What We Built
Email notifications are now sent via **Gmail SMTP** whenever a user transfers money. The email is sent asynchronously in the background, so users don't wait for delivery.

---

## 🔧 Step 1: Generate Gmail App Password

1. **Enable 2-Step Verification** (required):
   - Go to: https://myaccount.google.com/security
   - Click **2-Step Verification** → Follow the setup

2. **Generate App Password**:
   - Go to: https://myaccount.google.com/apppasswords
   - Select **Mail** as the app
   - Copy the **16-character password** (e.g., `abcd efgh ijkl mnop`)

3. **Update `.env` file**:
   ```env
   SMTP_HOST=smtp.gmail.com
   SMTP_PORT=587
   SMTP_USER=karkeepradeep654@gmail.com
   SMTP_PASSWORD=your_16_char_app_password_here
   SMTP_FROM=karkeepradeep654@gmail.com
   ```

   Replace `your_16_char_app_password_here` with your actual app password (remove spaces).

---

## 🧪 Step 2: Test Email Notifications

### Start the Server
```bash
cargo run
```

### Create Test Users
1. Register **User A** (e.g., `alice@example.com`)
2. Register **User B** (e.g., `bob@example.com`)

### Perform a Transfer
1. Log in as **User A**
2. Go to **Dashboard** → **Transfer**
3. Enter **User B's email** and amount (e.g., $50)
4. Click **Submit**

### Check Your Email
- **Recipient**: The email will be sent to User B's email address
- **From**: `karkeepradeep654@gmail.com`
- **Subject**: `MyFintechApp: Transfer Successful`
- **Body**: Plain text showing the transfer amount

**✨ No sandbox restrictions!** You can send to ANY email address.

---

## 🐛 Troubleshooting

### "Failed to send email" in server logs

**Check 1: App Password**
- Make sure you copied the 16-character password correctly (no spaces)
- Verify it's an **App Password**, not your regular Gmail password

**Check 2: 2-Step Verification**
- App Passwords only work if 2-Step Verification is enabled

**Check 3: SMTP Settings**
- Host: `smtp.gmail.com`
- Port: `587`
- User: Your full Gmail address

### Email not received

- Check the recipient's **spam folder**
- Verify the recipient email is correct
- Check Gmail's "Sent" folder to confirm it was sent

---

## 🔒 Security Notes

- **Never commit `.env`** to Git (it's already in `.gitignore`)
- App Passwords are safer than your main password
- Revoke app passwords you're not using: https://myaccount.google.com/apppasswords
