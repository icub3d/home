# Project: Home

A single-tenant family and home management system. Each user or family hosts their own instance of the system to manage their personal data in complete control.

## Architecture

The system consists of two main components:

### 1. Backend API
- **Language:** Rust (Axum)
- **Database:** PostgreSQL
- **Role:** Central data store, API, and OAuth handling.
- **Features:** Google OAuth, OpenWeather integration, background refresh jobs

### 2. Web Frontend
- **Framework:** React + TypeScript
- **Features:**
  - User registration and authentication
  - Dashboard/Overview mode (Kiosk) for displays
  - Widgets: Allowance balances, Weather, Calendar items, Chores, etc.

### 3. Mobile App (Future)
- **Framework:** React Native (Expo)
- **Platform:** Android (Primary), iOS (Capable)
- **Role:** Mobile management interface for the system.

## Infrastructure
- **Container Runtime:** Podman / Docker
- **Database:** PostgreSQL (running in a container)

## Goals
- Secure, private data management for a single household
- User-friendly interfaces for family members
- Simple self-hosting via containers or binary execution
