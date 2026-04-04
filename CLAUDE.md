# Architecture Guidelines

## Tauri/Rust Structure

```
src/                 # Rust backend
src-tauri/          # Tauri configuration
frontend/src/       # React frontend
src/
├── components/     # UI components
├── pages/          # Page components
├── hooks/          # React hooks
├── services/       # API calls
├── store/          # Zustand state
└── utils/          # Helpers
```

## Dependency Rules

- Frontend follows React patterns
- Backend in Rust (src-tauri)
- No direct DB access from frontend
- Use Tauri commands for backend calls

## Naming Conventions

- Components: `*.tsx`
- Pages: `*.tsx` in pages/
- Hooks: `use*.ts`
- Store: `*.store.ts`

## Before Generating Code

1. Identify frontend vs backend
2. Use Tauri commands for DB operations
3. Keep frontend stateless where possible
4. Run: `npm run lint` and `cargo check`