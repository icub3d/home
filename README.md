# Home - Family Management System

A comprehensive family and home management system designed to be self-hosted in your own home. Manage allowances, chores, calendars, and display family information on kiosk displays with full control over your data.

## 🌟 Features

- **👨‍👩‍👧‍👦 Family Management** - Create and manage family member accounts with role-based permissions
- **💰 Allowance Tracking** - Track allowance balances and transaction history for each family member
- **✅ Chore Management** - Create, assign, and track chores with completion status
- **📅 Calendar Integration** - Integrate Google Calendar and iCal feeds to display upcoming events
- **🌤️ Weather Widget** - Display current weather conditions using OpenWeather API
- **🖼️ Google Photos** - Connect Google Photos and display photo slideshows on kiosk displays
- **📺 Kiosk Display Mode** - Full-screen dashboard optimized for touchscreens with screensaver
- **🔐 Secure Authentication** - JWT-based authentication with admin/user roles
- **📱 Mobile Friendly** - Responsive design works on desktop, tablet, and mobile

## 🏗️ Architecture

The system consists of two main components:

### 1. Backend API (`/backend`)
- **Language:** Rust (Axum framework)
- **Database:** PostgreSQL
- **Features:**
  - RESTful API for all client operations
  - JWT authentication and authorization
  - Background tasks for photo/weather refresh
  - Database migrations with SQLx

### 2. Web Frontend (`/frontend`)
- **Framework:** React 19 + TypeScript
- **UI Library:** Material-UI (MUI)
- **State Management:** TanStack Query (React Query)
- **Features:**
  - Admin dashboard for family management
  - Settings and configuration interface
  - Kiosk mode for displays
  - Responsive design with dark/light themes

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.80+ ([Install Rust](https://rustup.rs/))
- **Node.js** 20+ and npm ([Install Node.js](https://nodejs.org/))
- **PostgreSQL** 15+
- **Google Cloud Project** (for Google Photos/Calendar) - See [GOOGLE_CLOUD_SETUP.md](GOOGLE_CLOUD_SETUP.md)
- **OpenWeather API Key** (free tier) - [Get API Key](https://openweathermap.org/api)

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd home
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   nano .env  # Edit with your values
   ```

   Required values:
   - `DATABASE_URL` - PostgreSQL connection string
   - `JWT_SECRET` - Generate with: `tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 64`
   - `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` - From Google Cloud Console
   - `OPENWEATHER_API_KEY` - From OpenWeather
   
   See [ENV_SETUP.md](ENV_SETUP.md) for detailed configuration.

3. **Install frontend dependencies**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

4. **Run database migrations and seed data**
   ```bash
   cd backend
   # Ensure your PostgreSQL database is running and DATABASE_URL is correct
   cargo run --bin seed
   cd ..
   ```

5. **Start the services** (recommended: use 2 terminal windows)

   **Terminal 1 - Backend:**
   ```bash
   cd backend
   cargo watch -x run
   # Or without watch: cargo run
   ```

   **Terminal 2 - Frontend:**
   ```bash
   cd frontend
   npm run dev
   ```

6. **Access the application**
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:4000

7. **Login with default credentials**
   - Username: `admin`
   - Password: `password`
   
   ⚠️ **Change this password immediately in production!**

## 📖 Usage

### Admin Features

1. **Family Management** (`/users`)
   - Add family members with names, birthdays, and roles
   - Assign admin privileges
   - Change passwords

2. **Allowance Management** (`/allowance`)
   - View all family member balances
   - Add credits (allowance payments)
   - Add debits (purchases/penalties)
   - View transaction history

3. **Chore Management** (`/chores`)
   - Create chores with names and rewards
   - Assign to family members
   - Mark chores as complete/incomplete
   - Track completion status

4. **Settings** (`/settings`)
   - Configure weather zip code
   - Connect Google account for Photos and Calendar
   - Add calendar feeds (iCal URLs or Google Calendar)
   - Manage display tokens for kiosk mode
   - Backup and restore system data

### Display/Kiosk Mode

1. **Create a display token**
   - Go to Settings → Display Tokens
   - Click "Create Token" and name it (e.g., "Living Room Display")
   - Copy the generated token

2. **Access display mode**
   - Navigate to `/display` on your kiosk device
   - Enter the display token
   - The display will show:
     - Weather widget
     - Upcoming calendar events
     - Allowance balances
     - Pending chores
     - Photo slideshow (if Google Photos connected)

3. **Features:**
   - Automatic screensaver after 1 minute of inactivity
   - Horizontal scrolling cards
   - Refreshes data every minute
   - No authentication required (token-based)

## 🔧 Development

### Project Structure

```
home/
├── backend/          # Rust backend API
│   ├── src/
│   │   ├── handlers/    # API route handlers
│   │   ├── middleware/  # Auth middleware
│   │   ├── models/      # Data models
│   │   ├── utils/       # Helper utilities
│   │   └── main.rs
│   └── migrations/   # SQLx database migrations
├── frontend/         # React frontend
│   └── src/
│       ├── api/         # API client
│       ├── components/  # React components
│       ├── context/     # React context providers
│       ├── pages/       # Page components
│       ├── types/       # TypeScript types
│       └── utils/       # Helper utilities
└── .env              # Environment configuration
```

### Development Tools

**Backend:**
- `cargo watch -x run` - Auto-reload on file changes
- `cargo test` - Run tests
- `cargo run --bin seed` - Seed database with admin user

**Frontend:**
- `npm run dev` - Development server with hot reload
- `npm run build` - Production build
- `npm run lint` - ESLint checking

### Database Migrations

Migrations are managed with SQLx and run automatically on startup.

To create a new migration:
```bash
cd backend
sqlx migrate add <migration_name>
# Edit the generated SQL file
sqlx migrate run
```

## 🐳 Production Deployment

### Option 1: Podman/Docker (Recommended)

Use the provided `docker-compose.yml` to spin up the entire stack including PostgreSQL.

```bash
docker-compose up -d
```

### Option 2: Raspberry Pi Setup

1. **Install dependencies**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Node.js
   curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
   sudo apt-get install -y nodejs

   # Install PostgreSQL
   sudo apt-get install postgresql
   ```

2. **Clone and configure**
   ```bash
   git clone <repository-url> ~/home
   cd ~/home
   cp .env.example .env
   nano .env  # Configure for production
   ```

3. **Build frontend**
   ```bash
   cd frontend
   npm install
   npm run build
   cd ..
   ```

4. **Build backend**
   ```bash
   cd backend
   cargo build --release
   cargo run --release --bin seed
   cd ..
   ```

5. **Run with systemd**
   Create service files in `/etc/systemd/system/`:
   - `home-backend.service`
   
   Enable and start:
   ```bash
   sudo systemctl enable home-backend
   sudo systemctl start home-backend
   ```

6. **Serve frontend**
   Use nginx or enable `SERVE_FRONTEND=true` in backend.

## 🔐 Security

- **JWT Authentication** - Secure token-based authentication
- **Password Hashing** - Argon2 password hashing
- **CORS Protection** - Configurable CORS origin
- **Rate Limiting** - Login endpoint rate limiting (2 req/sec)
- **Display Tokens** - Separate token system for kiosk displays
- **Environment Variables** - Sensitive config stored in `.env` (not committed)

### Security Best Practices

1. ⚠️ Change default admin password immediately
2. ⚠️ Generate a strong random JWT_SECRET (64+ characters)
3. ⚠️ Keep Google OAuth credentials secure
4. ⚠️ Use HTTPS in production
5. ⚠️ Regularly backup your database
6. ⚠️ Rotate display tokens periodically

## 🤝 Contributing

This is a personal/family project, but suggestions and improvements are welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is for personal/family use. Feel free to use and modify for your own needs.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum)
- Frontend powered by [React](https://react.dev/) and [Material-UI](https://mui.com/)
- Icons from [Material Icons](https://mui.com/material-ui/material-icons/)
- Weather data from [OpenWeatherMap](https://openweathermap.org/)
- Calendar parsing with [iCal.js](https://github.com/kewisch/ical.js/)

## 📚 Additional Documentation

- [ENV_SETUP.md](ENV_SETUP.md) - Detailed environment configuration guide
- [GOOGLE_CLOUD_SETUP.md](GOOGLE_CLOUD_SETUP.md) - Google Cloud Platform setup instructions
- [GEMINI.md](GEMINI.md) - Project context for AI assistance

## 🐛 Troubleshooting

### "Invalid token" error
- Ensure you've run `cargo run --bin seed` to create the admin user
- Check that JWT_SECRET is set in `.env`

### Google OAuth not working
- Verify GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET are correct
- Check that redirect URI matches in Google Cloud Console: `http://localhost:4000/api/google-photos/callback`

### Weather not loading
- Verify OPENWEATHER_API_KEY is valid
- Ensure zip code is set in Settings

### Display redirects to login
- Make sure you're accessing `/display` (not other protected routes)
- Verify the display token is valid
- Check that API is accessible

### Database locked error
- For PostgreSQL, check your connection limit and ensure the database server is running.

---

**Made with ❤️ for families**
