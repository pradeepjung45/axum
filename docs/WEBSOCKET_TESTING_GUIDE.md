# 🧪 Testing Real-Time WebSocket Notifications

## ✅ What We Built
Real-time notifications that instantly pop up on the recipient's screen when they receive money - no page refresh needed!

## 🚀 How to Test

### Step 1: Start the Server
```bash
cargo run
```

Wait for: `🌐 Server listening on http://0.0.0.0:3000`

### Step 2: Open Two Browser Windows
- **Window A**: Regular browser (Chrome/Firefox)
- **Window B**: Incognito/Private mode window

### Step 3: Create Two Users
1. **In Window A** - Register as User A (e.g., `alice@example.com`)
2. **In Window B** - Register as User B (e.g., `bob@example.com`)

### Step 4: Log Both Users In
1. **Window A**: Log in as Alice → Go to Dashboard
2. **Window B**: Log in as Bob → Go to Dashboard

### Step 5: Check WebSocket Connection
Open browser console (F12) in both windows. You should see:
```
✅ Connected to real-time notifications
```

### Step 6: Perform a Transfer
1. **In Window A (Alice)**: 
   - Click "Transfer"
   - Enter Bob's email: `bob@example.com`
   - Enter amount: `$50`
   - Click Submit

### Step 7: Watch Window B! 🎉
**Immediately** after submitting, you should see a green toast notification slide in from the right in **Window B (Bob's screen)**:

```
💰 You received $50 from a transfer!
```

The notification will disappear after 5 seconds.

---

## 🔍 What to Look For

### ✅ Success Indicators
1. Console log: `✅ Connected to real-time notifications`
2. Toast appears within **1 second** of transfer
3. Toast slides in from the right side
4. Toast has cash emoji 💰
5. Toast auto-dismisses after 5 seconds

### ❌ Troubleshooting

**"WebSocket connection failed"**
- Check server logs for authentication errors
- Verify you're logged in (auth_token cookie exists)
- Refresh the page

**"No notification appears"**
- Check server logs: Look for `📨 Sent notification to user {id}`
- Verify recipient user is online (Dashboard tab is open)
- Check if user is offline: Log says `User {id} is offline, skipping notification`

**Server Logs to Monitor**:
```bash
# When Bob connects:
✅ User {bob-uuid} connected to WebSocket

# When transfer happens:
📨 Sent notification to user {bob-uuid}

# When Bob disconnects:
❌ User {bob-uuid} disconnected from WebSocket
```

---

## 🎨 Demo Scenario

**The "Wow" Demo**:
1. Open 3 browser windows (Alice, Bob, Charlie)
2. Log all three in simultaneously
3. Have Alice send money to Bob  
   → Bob sees notification instantly
4. Have Bob send to Charlie  
   → Charlie sees notification
5. Close Charlie's tab  
   → Send from Alice to Charlie  
   → No notification (Charlie offline)  
   → Charlie logs back in and checks wallet (balance updated!)

---

## 🔧 Technical Details

**WebSocket URL**: `ws://localhost:3000/api/ws`  
**Authentication**: Uses `auth_token` cookie  
**Message Format**: Plain text string  
**Auto-Reconnect**: Not implemented (yet)  

**How It Works**:
1. On login → Browser connects to `/api/ws`
2. Server stores: `HashMap<UserId, WebSocket Connection>`
3. On transfer → Server looks up recipient in map
4. If online → Sends message through WebSocket
5. Browser receives → Shows toast notification

---

## 🎯 Next Steps (Optional Enhancements)

- Add different notification types (deposit, withdrawal)
- Show sender's name in notification
- Add sound effect
- Implement reconnection logic
- Store missed notifications in database
