# iQube Protocol Operations Manual

This website is built using [Docusaurus](https://docusaurus.io/), a modern static website generator with Mermaid diagram support.

## Installation

```bash
npm install
```

## Local Development

```bash
npm start
```

This command:
1. Automatically kills any existing processes on port 3001
2. Starts the Docusaurus development server on port 3001
3. Opens up a browser window at http://localhost:3001
4. Enables live reload for most changes without restarting the server

### Alternative Start Commands

```bash
# Start without killing existing processes (will prompt if port is busy)
npm run start-safe

# Manually kill processes on port 3001
npm run kill-port
```

### Port Configuration
- **Operations Manual**: http://localhost:3001 (this documentation site)
- **Aigent Z Application**: http://localhost:3000 (main application)
- **Ops Console**: http://localhost:3007 (standalone ops console)

### Mermaid Diagrams
This documentation includes interactive Mermaid diagrams that render automatically. All technical architecture diagrams are fully interactive and support both light and dark themes.

## Build

```bash
yarn build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Deployment

Using SSH:

```bash
USE_SSH=true yarn deploy
```

Not using SSH:

```bash
GIT_USER=<Your GitHub username> yarn deploy
```

If you are using GitHub pages for hosting, this command is a convenient way to build the website and push to the `gh-pages` branch.
