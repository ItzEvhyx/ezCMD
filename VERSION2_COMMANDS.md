# ezCMD Version 2: Command Library Plan

**Status:** Archived — imported for ezCMD 2.0.0
**Purpose:** Historical source for the v2 command library. The imported runtime data now lives in `data/*.toml`; implementation notes remain in `IMPLEMENTATIONV2.md`.

---

## 1. How the library is organized

v2 replaces the full command dump with a **table of contents**, so the library is organized in five levels:

| Level | What it is | Example | Shown by |
|:---|:---|:---|:---|
| **Category** | A family of related tools | Containers & DevOps | `ezcmd --help` |
| **Tool** | One tool or CLI, with a short flag name | Docker (`-docker`) | `ezcmd --help` |
| **Group** | A visual sub-heading inside a tool, for navigation only | Build & Release | `ezcmd --help -<tool>` |
| **Section** | One task, ordered so the commands can run top to bottom | Run the App | `ezcmd --help -<tool>` |
| **Commands** | The lines inside a section | `flutter run --release` | `ezcmd --help -<tool> -<section>` |

Groups exist only to keep long tool lists readable (Node.js & npm alone has 15 sections). They don't affect search or matching, and a section is always found by its title.

### Conventions

- **Section format** is the same as v1: a `# ---Title---` header, then `command --- note` lines with the notes aligned.
- **Placeholders** use `<angle-brackets>` with short names (`<pkg>`, `<container>`, `<file>`).
- **OS tags** prefix a line when it only applies to some systems: `[win]` Windows, `[linux]` Linux, `[mac]` macOS, `[unix]` Linux and macOS. Untagged lines work everywhere. When two tagged lines do the same job, they appear as a pair.
- **Tool platforms:** tools that only exist on some systems (for example systemd or CocoaPods) are marked, and the v1 rule applies: they are hidden on other systems.
- **Tool notes** record systematic differences, such as `gradlew` vs `./gradlew` on Windows.
- **Notes** are 60 characters or fewer, start with a verb, and have no trailing period.
- **Section size:** 2 to 12 commands. Anything bigger gets split.
- **Section titles** describe a task and reuse common names (Create a Project, Clean & Reset), so a query like `-clean` can offer every tool that has one through the `[1/2/3]` prompt.
- **Not in this file:** search aliases (the phrases people type, such as "remove container"). They are written during the v2 implementation, at least 3 per section.

---

## 2. Summary

| Category | Tools | Sections | Commands |
|:---|---:|---:|---:|
| JavaScript & Web | 10 | 47 | 243 |
| Mobile Development | 7 | 45 | 261 |
| Containers & DevOps | 7 | 41 | 250 |
| Cloud & Backend Services | 5 | 20 | 105 |
| Languages & Package Tools | 9 | 38 | 247 |
| Databases | 5 | 21 | 110 |
| Linux & System Tools | 12 | 47 | 292 |
| Package Managers | 6 | 18 | 85 |
| Version Control | 1 | 9 | 58 |
| **Total** | **62** | **286** | **1651** |

---

## 3. Table of contents

**JavaScript & Web**

- [Node.js & npm](#nodejs--npm) (`-npm`, 15 sections)
- [pnpm](#pnpm) (`-pnpm`, 5 sections)
- [Yarn](#yarn) (`-yarn`, 4 sections)
- [Bun](#bun) (`-bun`, 3 sections)
- [Next.js](#nextjs) (`-nextjs`, 4 sections)
- [React & Vite](#react--vite) (`-vite`, 3 sections)
- [TypeScript](#typescript) (`-typescript`, 4 sections)
- [ESLint & Prettier](#eslint--prettier) (`-eslint`, 2 sections)
- [Vercel CLI](#vercel-cli) (`-vercel`, 4 sections)
- [Netlify CLI](#netlify-cli) (`-netlify`, 3 sections)

**Mobile Development**

- [Flutter](#flutter) (`-flutter`, 16 sections)
- [Dart](#dart) (`-dart`, 4 sections)
- [React Native & Expo](#react-native--expo) (`-expo`, 8 sections)
- [Android (ADB & SDK)](#android-adb--sdk) (`-adb`, 7 sections)
- [Gradle](#gradle) (`-gradle`, 4 sections)
- [CocoaPods](#cocoapods) (`-pods`, 2 sections)
- [Xcode & Simulators](#xcode--simulators) (`-xcode`, 4 sections)

**Containers & DevOps**

- [Docker](#docker) (`-docker`, 9 sections)
- [Docker Compose](#docker-compose) (`-compose`, 5 sections)
- [Kubernetes (kubectl)](#kubernetes-kubectl) (`-kubectl`, 6 sections)
- [GitHub CLI (gh)](#github-cli-gh) (`-gh`, 7 sections)
- [SSH, SCP & rsync](#ssh-scp--rsync) (`-ssh`, 6 sections)
- [Terraform](#terraform) (`-terraform`, 5 sections)
- [Nginx & Certbot](#nginx--certbot) (`-nginx`, 3 sections)

**Cloud & Backend Services**

- [Firebase CLI](#firebase-cli) (`-firebase`, 5 sections)
- [Supabase CLI](#supabase-cli) (`-supabase`, 5 sections)
- [AWS CLI](#aws-cli) (`-aws`, 4 sections)
- [Google Cloud CLI (gcloud)](#google-cloud-cli-gcloud) (`-gcloud`, 3 sections)
- [Azure CLI](#azure-cli) (`-az`, 3 sections)

**Languages & Package Tools**

- [Python & pip](#python--pip) (`-python`, 7 sections)
- [Django, Flask & FastAPI](#django-flask--fastapi) (`-pyweb`, 3 sections)
- [uv](#uv) (`-uv`, 4 sections)
- [Poetry](#poetry) (`-poetry`, 3 sections)
- [Conda](#conda) (`-conda`, 3 sections)
- [Rust & Cargo](#rust--cargo) (`-cargo`, 8 sections)
- [Go](#go) (`-go`, 6 sections)
- [Java](#java) (`-java`, 2 sections)
- [Maven](#maven) (`-maven`, 2 sections)

**Databases**

- [PostgreSQL (psql)](#postgresql-psql) (`-postgres`, 5 sections)
- [MySQL & MariaDB](#mysql--mariadb) (`-mysql`, 4 sections)
- [SQLite](#sqlite) (`-sqlite`, 4 sections)
- [MongoDB](#mongodb) (`-mongodb`, 4 sections)
- [Redis](#redis) (`-redis`, 4 sections)

**Linux & System Tools**

- [Permissions & Users](#permissions--users) (`-perms`, 4 sections)
- [Find & Search Text](#find--search-text) (`-find`, 3 sections)
- [sed, awk & Text Tools](#sed-awk--text-tools) (`-sed`, 4 sections)
- [Archives (tar & zip)](#archives-tar--zip) (`-archive`, 4 sections)
- [systemd & journalctl](#systemd--journalctl) (`-systemd`, 5 sections)
- [Cron](#cron) (`-cron`, 3 sections)
- [Disk & Storage](#disk--storage) (`-disk`, 4 sections)
- [Shell Productivity (Bash)](#shell-productivity-bash) (`-bash`, 4 sections)
- [tmux](#tmux) (`-tmux`, 3 sections)
- [Vim](#vim) (`-vim`, 5 sections)
- [nano](#nano) (`-nano`, 3 sections)
- [curl & wget](#curl--wget) (`-curl`, 5 sections)

**Package Managers**

- [apt (Debian & Ubuntu)](#apt-debian--ubuntu) (`-apt`, 3 sections)
- [pacman & paru (Arch)](#pacman--paru-arch) (`-pacman`, 3 sections)
- [dnf (Fedora & RHEL)](#dnf-fedora--rhel) (`-dnf`, 3 sections)
- [Homebrew](#homebrew) (`-brew`, 4 sections)
- [winget](#winget) (`-winget`, 3 sections)
- [Flatpak & Snap](#flatpak--snap) (`-flatpak`, 2 sections)

**Version Control**

- [Git (additions to v1)](#git-additions-to-v1) (`-git`, 9 sections)

---

## 4. Command library

## JavaScript & Web

### Node.js & npm

**Flag:** `-npm`  |  **Also matches:** node, nodejs, npx, nvm  |  **Platforms:** Windows, Linux, macOS

> **Note:** nvm is a separate install: nvm-sh on Linux and macOS, nvm-windows on Windows

#### Setup & Projects

```text
# ---Check Versions & Run Files---
node -v           --- Prints the Node.js version
npm -v            --- Prints the npm version
node <file.js>    --- Runs a JavaScript file
node -e "<code>"  --- Runs a one-line script
node              --- Opens the interactive Node REPL

# ---Create a Project---
npm init                    --- Starts the package.json wizard
npm init -y                 --- Creates package.json with default answers
npm create vite@latest      --- Scaffolds a new Vite project
npx create-next-app@latest  --- Scaffolds a new Next.js app
npx create-expo-app@latest  --- Scaffolds a new Expo app

# ---Node Versions (nvm)---
[unix] nvm install --lts            --- Installs the latest LTS Node version
[win] nvm install lts               --- Installs the latest LTS Node version
nvm install <version>               --- Installs a specific Node version
nvm use <version>                   --- Switches to an installed version
[unix] nvm ls                       --- Lists installed versions
[win] nvm list                      --- Lists installed versions
[unix] nvm alias default <version>  --- Sets the default version for new terminals
```

#### Dependencies

```text
# ---Install Dependencies---
npm install                  --- Installs everything listed in package.json
npm ci                       --- Installs exactly what package-lock.json says
npm install <pkg>            --- Adds a package to dependencies
npm install -D <pkg>         --- Adds a package to devDependencies
npm install <pkg>@<version>  --- Installs a specific version
npm install --omit=dev       --- Skips devDependencies for production

# ---Update & Remove Packages---
npm uninstall <pkg>       --- Removes a package
npm outdated              --- Lists packages with newer versions
npm update                --- Updates packages within allowed ranges
npm install <pkg>@latest  --- Upgrades a package to its newest version
npx npm-check-updates -u  --- Rewrites package.json with the latest versions
npm prune                 --- Removes packages not listed in package.json

# ---Global Packages---
npm install -g <pkg>    --- Installs a package globally
npm uninstall -g <pkg>  --- Removes a global package
npm list -g --depth=0   --- Lists globally installed packages
npm root -g             --- Prints the global packages folder
npm config get prefix   --- Prints where global installs go

# ---Inspect Packages---
npm list --depth=0       --- Lists top-level installed packages
npm explain <pkg>        --- Shows why a package is installed
npm view <pkg> version   --- Prints a package's latest version
npm view <pkg> versions  --- Lists every published version
npm docs <pkg>           --- Opens a package's documentation
npm repo <pkg>           --- Opens a package's repository

# ---Link Local Packages---
npm link            --- Links the current package globally
npm link <pkg>      --- Uses a linked package in this project
npm unlink <pkg>    --- Removes the link
npm install <path>  --- Installs a local folder as a dependency
```

#### Scripts & Running

```text
# ---Run Scripts---
npm run                     --- Lists the scripts in package.json
npm run <script>            --- Runs a script from package.json
npm start                   --- Runs the start script
npm test                    --- Runs the test script
npm run <script> -- <args>  --- Passes extra arguments to a script
npm run build               --- Runs the build script

# ---Run Packages Without Installing (npx)---
npx <pkg>               --- Runs a package's command without installing
npx <pkg>@latest        --- Runs the newest version of a package
npx -y <pkg>            --- Skips the install confirmation prompt
npx -p <pkg> <command>  --- Runs a command that has a different name

# ---Workspaces (Monorepos)---
npm init -w <path>               --- Creates a new workspace package
npm install -w <workspace>       --- Installs in one workspace
npm run <script> -w <workspace>  --- Runs a script in one workspace
npm run <script> --workspaces    --- Runs a script in every workspace
```

#### Publishing & Maintenance

```text
# ---Publish a Package---
npm login                    --- Logs in to the npm registry
npm whoami                   --- Prints the logged-in user
npm version patch            --- Bumps the patch version and tags it
npm version minor            --- Bumps the minor version and tags it
npm version major            --- Bumps the major version and tags it
npm pack                     --- Creates the tarball that would be published
npm publish --dry-run        --- Previews what would be published
npm publish                  --- Publishes the package
npm publish --access public  --- Publishes a scoped package publicly

# ---Security Audit---
npm audit              --- Reports known vulnerabilities
npm audit fix          --- Applies safe automatic fixes
npm audit fix --force  --- Applies fixes that may break things
npm audit --omit=dev   --- Audits production dependencies only
npm audit signatures   --- Verifies package signatures

# ---Cache & Clean Reinstall---
npm cache verify                              --- Checks the cache for corruption
npm cache clean --force                       --- Wipes the npm cache
[unix] rm -rf node_modules package-lock.json  --- Deletes installed packages and the lockfile
[win] rmdir /s /q node_modules                --- Deletes the node_modules folder
[win] del package-lock.json                   --- Deletes the lockfile
npm install                                   --- Reinstalls everything from scratch
npm dedupe                                    --- Flattens duplicate dependencies

# ---Config & Registry---
npm config list                --- Shows your npm configuration
npm config get registry        --- Prints the current registry URL
npm config set registry <url>  --- Switches the registry
npm config delete <key>        --- Removes a setting
npm config edit                --- Opens the npm config file
```

### pnpm

**Flag:** `-pnpm`  |  **Also matches:** pnpx  |  **Platforms:** Windows, Linux, macOS

#### Packages

```text
# ---Install & Add Packages---
pnpm install                    --- Installs everything in package.json
pnpm install --frozen-lockfile  --- Fails if the lockfile needs changes
pnpm add <pkg>                  --- Adds a dependency
pnpm add -D <pkg>               --- Adds a dev dependency
pnpm add -g <pkg>               --- Installs a package globally

# ---Update & Remove Packages---
pnpm remove <pkg>     --- Removes a package
pnpm update           --- Updates packages within allowed ranges
pnpm update --latest  --- Updates to the newest versions
pnpm outdated         --- Lists packages with newer versions
pnpm why <pkg>        --- Shows why a package is installed
```

#### Running & Maintenance

```text
# ---Run Scripts & Packages---
pnpm run <script>  --- Runs a script from package.json
pnpm dev           --- Runs the dev script
pnpm dlx <pkg>     --- Runs a package without installing it
pnpm create vite   --- Scaffolds a project from a template

# ---Workspaces (Monorepos)---
pnpm --filter <pkg> <command>        --- Runs a command in one workspace package
pnpm -r <command>                    --- Runs a command in every workspace package
pnpm add <pkg> --filter <workspace>  --- Adds a dependency to one workspace

# ---Store & Cleanup---
pnpm store path       --- Prints the global store location
pnpm store prune      --- Removes unused packages from the store
pnpm install --force  --- Refetches and relinks all packages
```

### Yarn

**Flag:** `-yarn`  |  **Also matches:** yarnpkg  |  **Platforms:** Windows, Linux, macOS

> **Note:** Classic (v1) and Berry (v2+) differ. Lines are labeled where they do

#### Packages

```text
# ---Install & Add Packages---
yarn                            --- Installs everything in package.json
yarn add <pkg>                  --- Adds a dependency
yarn add -D <pkg>               --- Adds a dev dependency
yarn install --frozen-lockfile  --- Classic: fails if the lockfile changes
yarn install --immutable        --- Berry: fails if the lockfile changes

# ---Update & Remove Packages---
yarn remove <pkg>  --- Removes a package
yarn upgrade       --- Classic: updates packages
yarn up <pkg>      --- Berry: upgrades a package
yarn outdated      --- Classic: lists packages with newer versions
yarn why <pkg>     --- Shows why a package is installed
```

#### Running & Maintenance

```text
# ---Scripts & Workspaces---
yarn <script>                            --- Runs a script from package.json
yarn run                                 --- Lists the available scripts
yarn workspace <name> <command>          --- Runs a command in one workspace
yarn workspaces foreach -A run <script>  --- Berry: runs a script in every workspace
yarn dlx <pkg>                           --- Berry: runs a package without installing
yarn create next-app                     --- Classic: scaffolds a project from a template
yarn dlx create-next-app                 --- Berry: scaffolds a project from a template

# ---Global Packages & Cache---
yarn global add <pkg>    --- Classic: installs a package globally
yarn cache clean         --- Clears the Yarn cache
yarn set version stable  --- Berry: upgrades Yarn itself
yarn --version           --- Prints the Yarn version
```

### Bun

**Flag:** `-bun`  |  **Also matches:** bunx  |  **Platforms:** Windows, Linux, macOS

#### Everything Bun

```text
# ---Create & Install---
bun init          --- Creates a new Bun project
bun install       --- Installs everything in package.json
bun add <pkg>     --- Adds a dependency
bun add -d <pkg>  --- Adds a dev dependency
bun remove <pkg>  --- Removes a package
bun update        --- Updates packages

# ---Run Code & Scripts---
bun run <file.ts>          --- Runs a TypeScript or JavaScript file
bun run <script>           --- Runs a script from package.json
bun --watch run <file.ts>  --- Reruns the file when it changes
bunx <pkg>                 --- Runs a package without installing
bun create <template>      --- Scaffolds a project from a template

# ---Test, Build & Upgrade---
bun test                                         --- Runs the test suite
bun build <entry.ts> --outdir <dir>              --- Bundles code into a folder
bun build <entry.ts> --compile --outfile <name>  --- Compiles to a standalone executable
bun upgrade                                      --- Updates Bun itself
bun pm cache rm                                  --- Clears the Bun cache
```

### Next.js

**Flag:** `-nextjs`  |  **Also matches:** next, vercel-next  |  **Platforms:** Windows, Linux, macOS

#### Everything Next.js

```text
# ---Create an App---
npx create-next-app@latest                             --- Scaffolds an app with prompts
npx create-next-app@latest <name> --yes                --- Creates an app using the recommended defaults
npx create-next-app@latest <name> --ts                 --- Creates a TypeScript app (the default)
npx create-next-app@latest <name> --no-ts              --- Creates a JavaScript app
npx create-next-app@latest <name> --use-pnpm           --- Creates an app using pnpm (or --use-bun)
npx create-next-app@latest <name> --example <example>  --- Starts from an official example

# ---Dev, Build & Start---
npm run dev                                            --- Starts the dev server
npx next dev -p <port>                                 --- Starts the dev server on another port
npm run build                                          --- Creates a production build
npm run start                                          --- Serves the production build
npx next start -p <port>                               --- Serves the production build on another port
npx next build --webpack                               --- Builds with Webpack instead of Turbopack
npx next info                                          --- Prints environment info for bug reports
npm install next@latest react@latest react-dom@latest  --- Upgrades Next.js and React

# ---Clean & Reset---
[unix] rm -rf .next           --- Deletes the Next.js build cache
[win] rmdir /s /q .next       --- Deletes the Next.js build cache
npx next telemetry --disable  --- Turns off anonymous telemetry
npx next telemetry --enable   --- Turns anonymous telemetry back on

# ---Deploy---
npm install -g vercel  --- Installs the Vercel CLI
vercel                 --- Deploys a preview build
vercel --prod          --- Deploys to production
```

### React & Vite

**Flag:** `-vite`  |  **Also matches:** react, vitest  |  **Platforms:** Windows, Linux, macOS

#### Everything Vite

```text
# ---Create a Project---
npm create vite@latest                                --- Scaffolds a project with prompts
npm create vite@latest <name> -- --template react     --- Creates a React project
npm create vite@latest <name> -- --template react-ts  --- Creates a React and TypeScript project
pnpm create vite                                      --- Scaffolds a project using pnpm
yarn create vite                                      --- Scaffolds a project using Yarn
bun create vite                                       --- Scaffolds a project using Bun

# ---Dev, Build & Preview---
npm run dev                   --- Starts the Vite dev server
npm run build                 --- Builds for production
npm run preview               --- Serves the production build locally
npx vite --host               --- Exposes the dev server on your network
npx vite --port <port>        --- Starts the dev server on a chosen port
npx vite build --mode <mode>  --- Builds using a specific env mode

# ---Testing (Vitest)---
npm install -D vitest                   --- Installs Vitest
npx vitest                              --- Runs tests in watch mode
npx vitest run                          --- Runs tests once
npm install -D @vitest/coverage-v8      --- Installs the coverage provider
npx vitest run --coverage               --- Runs tests with a coverage report
npx vitest <file>                       --- Runs tests in one file
```

### TypeScript

**Flag:** `-typescript`  |  **Also matches:** ts, tsc, tsx  |  **Platforms:** Windows, Linux, macOS

> **Note:** TypeScript 7 ships the native Go compiler as tsc. The @typescript/typescript6 package provides the 6.0 compiler as tsc6. In TypeScript 7, passing file paths in a folder that has a tsconfig.json fails unless you add --ignoreConfig

#### Everything TypeScript

```text
# ---Set Up TypeScript---
npm install -D typescript    --- Adds TypeScript to a project
npx tsc --init               --- Creates a tsconfig.json
npx tsc -v                   --- Prints the TypeScript version
npm install -D @types/node   --- Adds Node.js type definitions
npm install -D @types/<pkg>  --- Adds type definitions for a package

# ---Compile & Type-Check---
npx tsc                     --- Compiles the project
npx tsc --noEmit            --- Type-checks without writing files
npx tsc --watch             --- Recompiles when files change
npx tsc -p <tsconfig.json>  --- Compiles using a specific config
npx tsc --ignoreConfig <file.ts>  --- Compiles one file and ignores tsconfig.json

# ---Run TypeScript Directly---
npx tsx <file.ts>        --- Runs a TypeScript file with tsx
npx tsx watch <file.ts>  --- Reruns when the file changes
npx ts-node <file.ts>    --- Runs a TypeScript file with ts-node
bun run <file.ts>        --- Runs a TypeScript file with Bun

# ---Inspect the Setup---
npx tsc --showConfig                         --- Prints the resolved tsconfig
npx tsc --listFiles                          --- Lists every file in the compilation
npx tsc --declaration --emitDeclarationOnly  --- Generates only .d.ts files
```

### ESLint & Prettier

**Flag:** `-eslint`  |  **Also matches:** prettier, lint, format  |  **Platforms:** Windows, Linux, macOS

#### Everything Linting

```text
# ---ESLint---
npm init @eslint/config@latest  --- Sets up an ESLint config interactively
npx eslint .                    --- Lints the whole project
npx eslint . --fix              --- Fixes problems automatically
npx eslint <file>               --- Lints one file
npx eslint . --max-warnings 0   --- Fails on any warning

# ---Prettier---
npm install -D prettier                   --- Installs Prettier
npx prettier --write .                    --- Formats every file
npx prettier --check .                    --- Checks formatting without changing files
npx prettier --write <file>               --- Formats one file
npx prettier --write "src/**/*.{ts,tsx}"  --- Formats files matching a pattern
```

### Vercel CLI

**Flag:** `-vercel`  |  **Also matches:** vercel-cli  |  **Platforms:** Windows, Linux, macOS

#### Everything Vercel

```text
# ---Log In & Link---
npm install -g vercel  --- Installs the Vercel CLI
vercel login           --- Logs in to Vercel
vercel link            --- Links the folder to a Vercel project
vercel whoami          --- Prints the logged-in account
vercel logout          --- Logs out of Vercel

# ---Deploy---
vercel                           --- Deploys a preview build
vercel --prod                    --- Deploys to production
vercel dev                       --- Runs the project locally like Vercel
vercel ls                        --- Lists recent deployments
vercel rollback                  --- Rolls production back to the previous deployment
vercel promote <deployment-url>  --- Makes a deployment the production one

# ---Environment Variables---
vercel env ls               --- Lists environment variables
vercel env add <name>       --- Adds an environment variable
vercel env pull .env.local  --- Downloads variables into .env.local
vercel pull                 --- Downloads project settings and variables
vercel env rm <name>        --- Removes an environment variable

# ---Logs & Domains---
vercel logs <deployment-url>           --- Shows runtime logs for a deployment
vercel inspect <deployment-url>        --- Shows deployment details
vercel domains ls                      --- Lists your domains
vercel domains add <domain> <project>  --- Adds a domain to a project
vercel remove <name>                   --- Deletes a project or deployment
```

### Netlify CLI

**Flag:** `-netlify`  |  **Also matches:** netlify-cli  |  **Platforms:** Windows, Linux, macOS

#### Everything Netlify

```text
# ---Log In & Link---
npm install -g netlify-cli  --- Installs the Netlify CLI
netlify login               --- Logs in to Netlify
netlify init                --- Creates or connects a site
netlify link                --- Links the folder to an existing site
netlify status              --- Shows the current site and login

# ---Develop & Deploy---
netlify dev            --- Runs the site locally with Netlify features
netlify build          --- Runs the Netlify build locally
netlify deploy         --- Deploys a draft preview
netlify deploy --prod  --- Deploys to production
netlify open           --- Opens the site admin in the browser

# ---Environment & Functions---
netlify env:list                 --- Lists environment variables
netlify env:set <KEY> <value>    --- Sets an environment variable
netlify env:import <file>        --- Imports variables from a .env file
netlify functions:create         --- Creates a new serverless function
netlify functions:invoke <name>  --- Calls a function locally
```

---

## Mobile Development

### Flutter

**Flag:** `-flutter`  |  **Also matches:** flutter-sdk, pub  |  **Platforms:** Windows, Linux, macOS

#### Setup & Projects

```text
# ---Check Your Setup---
flutter doctor          --- Checks your Flutter installation
flutter doctor -v       --- Shows detailed setup information
flutter --version       --- Prints the Flutter and Dart versions
flutter upgrade         --- Updates Flutter to the latest version
flutter channel         --- Lists Flutter channels
flutter channel stable  --- Switches to the stable channel

# ---Create a Project---
flutter create <name>                          --- Creates a new Flutter app
flutter create --org <com.example> <name>      --- Sets the app's organization ID
flutter create --platforms=android,ios <name>  --- Creates an app for chosen platforms
flutter create --template=package <name>       --- Creates a reusable Flutter package
flutter create .                               --- Adds missing platform folders to this project
flutter create . --platforms=web               --- Adds web support to an existing project
```

#### Running

```text
# ---Run the App---
flutter devices                          --- Lists connected devices and emulators
flutter run                              --- Runs the app in debug mode
flutter run -d <device-id>               --- Runs on a specific device
flutter run --profile                    --- Runs in profile mode for performance testing
flutter run --release                    --- Runs in release mode
flutter run -t <lib/main_dev.dart>       --- Runs a different entry file
flutter run --flavor <name>              --- Runs a specific build flavor
flutter run --dart-define=<KEY>=<value>  --- Passes a compile-time variable

# ---Emulators---
flutter emulators                         --- Lists available emulators
flutter emulators --launch <id>           --- Starts an emulator
flutter emulators --create                --- Creates an Android emulator based on a Pixel device
flutter emulators --create --name <name>  --- Creates an emulator with a chosen name
flutter devices --machine                 --- Lists devices as JSON

# ---Run on Web & Desktop---
flutter run -d chrome         --- Runs the app in Chrome
flutter run -d chrome --wasm  --- Runs the app in Chrome using WebAssembly
flutter run -d web-server     --- Serves the app without opening a browser
flutter run -d windows        --- Runs the app as a Windows desktop app
flutter run -d linux          --- Runs the app as a Linux desktop app
flutter run -d macos          --- Runs the app as a macOS desktop app

# ---Hot Reload Keys (While Running)---
r  --- Hot reloads the app
R  --- Hot restarts the app
h  --- Lists all available keys
d  --- Detaches and leaves the app running
c  --- Clears the terminal
s  --- Saves a screenshot
b  --- Toggles light and dark mode
i  --- Toggles the widget inspector
w  --- Dumps the widget tree to the console
o  --- Simulates a different operating system
q  --- Quits the app
```

#### Packages & Code Generation

```text
# ---Manage Packages---
flutter pub get                       --- Downloads the packages in pubspec.yaml
flutter pub add <pkg>                 --- Adds a package to pubspec.yaml
flutter pub add dev:<pkg>             --- Adds a dev dependency
flutter pub remove <pkg>              --- Removes a package
flutter pub upgrade                   --- Upgrades packages within version limits
flutter pub upgrade --major-versions  --- Upgrades and rewrites version constraints
flutter pub outdated                  --- Lists packages with newer versions
flutter pub deps                      --- Prints the dependency tree

# ---Code Generation (build_runner)---
dart run build_runner build                               --- Generates code once
dart run build_runner build --delete-conflicting-outputs  --- Regenerates and resolves conflicts
dart run build_runner watch                               --- Regenerates when files change
dart run build_runner clean                               --- Deletes generated files

# ---Icons, Splash & Localization---
dart run flutter_launcher_icons        --- Generates app icons
dart run flutter_native_splash:create  --- Generates the splash screen
flutter gen-l10n                       --- Generates localization files
```

#### Build & Release

```text
# ---Build for Android---
flutter build apk                                       --- Builds a release APK
flutter build apk --debug                               --- Builds a debug APK
flutter build apk --split-per-abi                       --- Builds smaller APKs per CPU type
flutter build appbundle                                 --- Builds an AAB for the Play Store
flutter build apk --obfuscate --split-debug-info=<dir>  --- Obfuscates code and saves debug symbols
flutter install                                         --- Installs the built app on a device

# ---Build for iOS (macOS Only)---
[mac] flutter build ios                --- Builds the iOS app
[mac] flutter build ios --no-codesign  --- Builds without code signing
[mac] flutter build ipa                --- Builds an archive for the App Store
[mac] open ios/Runner.xcworkspace      --- Opens the iOS project in Xcode

# ---Build for Web & Desktop---
flutter build web         --- Builds the web app
flutter build web --wasm  --- Builds the web app as WebAssembly
flutter build windows     --- Builds a Windows desktop app
flutter build linux       --- Builds a Linux desktop app
flutter build macos       --- Builds a macOS desktop app

# ---Android Signing Keys---
keytool -genkey -v -keystore <keystore.jks> -keyalg RSA -keysize 2048 -validity 10000 -alias <alias>  --- Creates a release keystore
keytool -list -v -keystore <keystore.jks> -alias <alias>                                              --- Shows the SHA-1 and SHA-256 fingerprints
[unix] keytool -list -v -alias androiddebugkey -keystore ~/.android/debug.keystore                    --- Shows debug fingerprints (password: android)
[win] keytool -list -v -alias androiddebugkey -keystore %USERPROFILE%\.android\debug.keystore         --- Shows debug fingerprints (password: android)
```

#### Quality & Troubleshooting

```text
# ---Test & Analyze---
flutter analyze                --- Checks the code for errors and lint issues
flutter test                   --- Runs all unit and widget tests
flutter test <file>            --- Runs one test file
flutter test --coverage        --- Runs tests with coverage output
flutter test integration_test  --- Runs integration tests
dart format .                  --- Formats all Dart files
dart fix --apply               --- Applies automated lint fixes

# ---Clean & Reset---
flutter clean                      --- Deletes build artifacts and caches
flutter pub get                    --- Re-downloads packages after a clean
flutter pub cache repair           --- Repairs the pub cache
flutter precache                   --- Downloads platform binaries ahead of time
flutter doctor --android-licenses  --- Accepts the Android SDK licenses

# ---Debug & Logs---
flutter logs           --- Streams logs from a running app
flutter attach         --- Attaches to an app that is already running
flutter screenshot     --- Saves a screenshot of the running app
flutter run --verbose  --- Shows detailed output while running
dart devtools          --- Opens Flutter DevTools
```

### Dart

**Flag:** `-dart`  |  **Also matches:** dart-sdk  |  **Platforms:** Windows, Linux, macOS

#### Everything Dart

```text
# ---Create & Run---
dart create <name>             --- Creates a new Dart project
dart create -t console <name>  --- Creates a console app
dart run                       --- Runs the project's main file
dart run <file.dart>           --- Runs a Dart file
dart --version                 --- Prints the Dart version

# ---Manage Packages---
dart pub get                    --- Downloads packages
dart pub add <pkg>              --- Adds a package
dart pub add dev:<pkg>          --- Adds a dev dependency
dart pub remove <pkg>           --- Removes a package
dart pub upgrade                --- Upgrades packages
dart pub outdated               --- Lists outdated packages
dart pub global activate <pkg>  --- Installs a command-line package globally

# ---Compile---
dart compile exe <file.dart>           --- Compiles to a native executable
dart compile js <file.dart>            --- Compiles to JavaScript
dart compile aot-snapshot <file.dart>  --- Compiles to an AOT snapshot
dart compile kernel <file.dart>        --- Compiles to a kernel module

# ---Test, Analyze & Format---
dart test           --- Runs tests
dart analyze        --- Checks the code for issues
dart format .       --- Formats the code
dart fix --dry-run  --- Previews automated fixes
dart fix --apply    --- Applies automated fixes
dart doc            --- Generates API documentation
```

### React Native & Expo

**Flag:** `-expo`  |  **Also matches:** react-native, rn, eas  |  **Platforms:** Windows, Linux, macOS

#### Expo

```text
# ---Create an Expo App---
npx create-expo-app@latest <name>                              --- Creates a new Expo app
npx create-expo-app@latest <name> --template blank-typescript  --- Creates a blank TypeScript app
npx expo install <pkg>                                         --- Installs a version compatible with your SDK
npx expo install --fix                                         --- Fixes mismatched package versions
npx expo-doctor                                                --- Diagnoses common project problems

# ---Run & Develop (Expo)---
npx expo start             --- Starts the dev server
npx expo start --clear     --- Starts with a cleared cache
npx expo start --tunnel    --- Starts a tunnel for devices on other networks
npx expo run:android       --- Builds and runs the Android app
npx expo run:ios           --- Builds and runs the iOS app
npx expo prebuild          --- Generates the native android and ios folders
npx expo prebuild --clean  --- Regenerates the native folders from scratch

# ---Dev Server Keys (While Running)---
a  --- Opens the app on Android
i  --- Opens the app on the iOS simulator
w  --- Opens the app in the browser
r  --- Reloads the app
m  --- Toggles the developer menu
j  --- Opens the debugger
```

#### Builds & Updates (EAS)

```text
# ---Set Up EAS---
npm install -g eas-cli  --- Installs the EAS CLI
eas login               --- Logs in to your Expo account
eas whoami              --- Prints the logged-in account
eas init                --- Links the project to EAS
eas build:configure     --- Creates eas.json for builds

# ---Build & Submit---
eas build --platform android   --- Builds the Android app in the cloud
eas build --platform ios       --- Builds the iOS app in the cloud
eas build --platform all       --- Builds for both platforms
eas build --profile <profile>  --- Builds using a profile from eas.json
eas build --local              --- Runs the build on your machine
eas submit --platform android  --- Submits the build to Google Play
eas submit --platform ios      --- Submits the build to App Store Connect

# ---OTA Updates & Builds---
eas update --branch <branch> --message "<message>"  --- Publishes an over-the-air update
eas update --auto                                   --- Uses the Git branch and last commit message
eas update:rollback                                 --- Rolls back to a previous update
eas update:list                                     --- Lists published updates
eas channel:list                                    --- Lists update channels
eas build:list                                      --- Lists your builds
```

#### React Native CLI

```text
# ---Create & Run---
npx @react-native-community/cli@latest init <name>  --- Creates a new React Native app
npx react-native start                              --- Starts the Metro bundler
npx react-native run-android                        --- Builds and runs on Android
npx react-native run-ios                            --- Builds and runs on iOS
npx react-native start --reset-cache                --- Starts Metro with a cleared cache
npx react-native doctor                             --- Checks your development environment

# ---Clean & Fix Builds---
[unix] cd android && ./gradlew clean       --- Cleans the Android build
[win] cd android && gradlew clean          --- Cleans the Android build
[mac] cd ios && pod install                --- Reinstalls iOS native pods
[unix] rm -rf node_modules && npm install  --- Reinstalls JS dependencies from scratch
[unix] watchman watch-del-all              --- Resets Watchman's file watches
npx react-native info                      --- Prints environment details for bug reports
```

### Android (ADB & SDK)

**Flag:** `-adb`  |  **Also matches:** android, sdkmanager, avdmanager, emulator  |  **Platforms:** Windows, Linux, macOS

#### ADB

```text
# ---Connect to Devices---
adb devices                --- Lists connected devices
adb pair <ip>:<port>       --- Pairs with a device over wireless debugging
adb connect <ip>:<port>    --- Connects to a device over Wi-Fi
adb disconnect             --- Disconnects wireless devices
adb tcpip 5555             --- Switches a USB device to TCP/IP mode
adb -s <serial> <command>  --- Runs a command on one specific device
adb kill-server            --- Stops the ADB server
adb start-server           --- Starts the ADB server

# ---Install & Manage Apps---
adb install <app.apk>                                                --- Installs an APK
adb install -r <app.apk>                                             --- Reinstalls an APK and keeps its data
adb uninstall <package>                                              --- Uninstalls an app
adb shell pm list packages -3                                        --- Lists installed third-party apps
adb shell pm clear <package>                                         --- Clears an app's data
adb shell am force-stop <package>                                    --- Force-stops an app
adb shell monkey -p <package> -c android.intent.category.LAUNCHER 1  --- Launches an app by package name

# ---Logs & Debugging---
adb logcat                 --- Streams device logs
adb logcat -c              --- Clears the log buffer
adb logcat "*:E"           --- Shows errors only
adb logcat -d > <log.txt>  --- Saves the current logs to a file
adb bugreport              --- Collects a full bug report

# ---Files & Screen---
adb push <local-path> <device-path>      --- Copies a file to the device
adb pull <device-path> <local-path>      --- Copies a file from the device
adb shell ls /sdcard                     --- Lists files on shared storage
adb shell screencap /sdcard/screen.png   --- Takes a screenshot on the device
adb shell screenrecord /sdcard/demo.mp4  --- Records the device screen

# ---Device Info & Control---
adb reboot                                  --- Restarts the device
adb reboot bootloader                       --- Restarts into the bootloader
adb reboot recovery                         --- Restarts into recovery
adb shell getprop ro.build.version.release  --- Prints the Android version
adb shell wm size                           --- Prints the screen resolution
adb shell dumpsys battery                   --- Shows battery information
```

#### SDK & Emulator

```text
# ---SDK Manager---
sdkmanager --list                     --- Lists available SDK packages
sdkmanager "platform-tools"           --- Installs or updates platform tools
sdkmanager "platforms;android-<api>"  --- Installs an Android platform
sdkmanager --licenses                 --- Accepts all SDK licenses
sdkmanager --update                   --- Updates installed packages

# ---Virtual Devices (AVD)---
emulator -list-avds                     --- Lists your virtual devices
emulator -avd <name>                    --- Starts a virtual device
emulator -avd <name> -wipe-data         --- Starts with factory-reset data
emulator -avd <name> -no-snapshot-load  --- Cold boots the emulator
avdmanager list avd                     --- Lists virtual devices with details
avdmanager delete avd -n <name>         --- Deletes a virtual device
```

### Gradle

**Flag:** `-gradle`  |  **Also matches:** gradlew, android-build  |  **Platforms:** Windows, Linux, macOS

> **Note:** On Windows CMD use gradlew instead of ./gradlew

#### Android Builds

```text
# ---Build & Install Android Apps---
./gradlew assembleDebug    --- Builds a debug APK
./gradlew assembleRelease  --- Builds a release APK
./gradlew bundleRelease    --- Builds a release AAB
./gradlew installDebug     --- Builds and installs the debug app
./gradlew clean            --- Deletes build outputs

# ---Android Checks & Reports---
./gradlew signingReport         --- Prints SHA-1 and SHA-256 fingerprints
./gradlew lint                  --- Runs Android lint
./gradlew test                  --- Runs unit tests
./gradlew connectedAndroidTest  --- Runs instrumented tests on a device
./gradlew app:dependencies      --- Prints the app's dependency tree
```

#### General Gradle

```text
# ---Build & Test---
gradle init           --- Creates a new Gradle project
gradle build          --- Compiles, tests, and packages the project
gradle build -x test  --- Builds without running tests
gradle test           --- Runs tests
gradle run            --- Runs the application
gradle clean          --- Deletes build outputs

# ---Inspect & Troubleshoot---
gradle tasks                         --- Lists available tasks
gradle dependencies                  --- Prints the dependency tree
gradle build --refresh-dependencies  --- Redownloads dependencies
gradle build --stacktrace            --- Shows full error stack traces
gradle --stop                        --- Stops all Gradle daemons
gradle wrapper                       --- Generates the Gradle wrapper files
```

### CocoaPods

**Flag:** `-pods`  |  **Also matches:** cocoapods, pod  |  **Platforms:** macOS

#### Everything CocoaPods

```text
# ---Install & Update Pods---
brew install cocoapods     --- Installs CocoaPods
pod init                   --- Creates a Podfile
pod install                --- Installs the pods in the Podfile
pod install --repo-update  --- Installs after updating the pod index
pod update                 --- Updates all pods
pod update <pod>           --- Updates one pod
pod outdated               --- Lists pods with newer versions

# ---Fix Pod Problems---
pod deintegrate        --- Removes CocoaPods from the project
pod cache clean --all  --- Clears the pod cache
pod repo update        --- Updates the local pod index
pod --version          --- Prints the CocoaPods version
```

### Xcode & Simulators

**Flag:** `-xcode`  |  **Also matches:** xcodebuild, simctl, ios-simulator  |  **Platforms:** macOS

#### Everything Xcode

```text
# ---Command Line Tools---
xcode-select --install       --- Installs the Xcode command line tools
xcode-select -p              --- Prints the active developer folder
sudo xcode-select -s <path>  --- Switches the active Xcode
xcodebuild -version          --- Prints the Xcode version
xcodebuild -showsdks         --- Lists available SDKs

# ---Build & Test---
xcodebuild -list                                                                    --- Lists schemes and targets
xcodebuild -scheme <scheme> build                                                   --- Builds a scheme
xcodebuild -scheme <scheme> -destination 'platform=iOS Simulator,name=<device>' test  --- Runs a scheme's tests on a simulator
xcodebuild -workspace <name>.xcworkspace -scheme <scheme> build                     --- Builds a workspace scheme
xcodebuild -showdestinations -scheme <scheme>                                       --- Lists the destinations a scheme can use
xcodebuild clean                                                                    --- Cleans build products

# ---Simulators (simctl)---
xcrun simctl list devices                     --- Lists simulators
xcrun simctl boot <udid>                      --- Starts a simulator
xcrun simctl shutdown all                     --- Shuts down all simulators
xcrun simctl erase all                        --- Resets all simulators
xcrun simctl install booted <app-path>        --- Installs an app on the booted simulator
xcrun simctl launch booted <bundle-id>        --- Launches an app
xcrun simctl openurl booted <url>             --- Opens a URL in the simulator
xcrun simctl io booted screenshot <file.png>  --- Takes a simulator screenshot
open -a Simulator                             --- Opens the Simulator app

# ---Clear Caches---
rm -rf ~/Library/Developer/Xcode/DerivedData  --- Clears Xcode build caches
xcrun simctl delete unavailable               --- Removes unusable simulators
rm -rf ~/Library/Caches/CocoaPods             --- Clears the CocoaPods cache
```

---

## Containers & DevOps

### Docker

**Flag:** `-docker`  |  **Also matches:** container, containers, dockerfile  |  **Platforms:** Windows, Linux, macOS

#### Containers

```text
# ---Check Docker & Log In---
docker --version  --- Prints the Docker version
docker info       --- Shows system-wide Docker information
docker login      --- Logs in to a registry
docker logout     --- Logs out of a registry
docker system df  --- Shows disk space used by Docker

# ---Run a Container---
docker run <image>                                  --- Runs a container from an image
docker run -d <image>                               --- Runs the container in the background
docker run -it <image> sh                           --- Runs the container with an interactive shell
docker run -p <host-port>:<container-port> <image>  --- Publishes a container port on the host
docker run -v <host-path>:<container-path> <image>  --- Mounts a host folder into the container
docker run --name <name> --rm <image>               --- Names the container and removes it on exit
docker run -e <KEY>=<value> <image>                 --- Sets an environment variable
docker run --env-file <file> <image>                --- Loads environment variables from a file

# ---Manage Containers---
docker ps                   --- Lists running containers
docker ps -a                --- Lists all containers, including stopped
docker stop <container>     --- Stops a running container
docker start <container>    --- Starts a stopped container
docker restart <container>  --- Restarts a container
docker rm <container>       --- Removes a stopped container
docker rm -f <container>    --- Force-removes a container
docker rename <old> <new>   --- Renames a container

# ---Inspect & Debug---
docker logs <container>                    --- Shows a container's logs
docker logs -f <container>                 --- Follows a container's logs live
docker logs --tail 100 <container>         --- Shows the last 100 log lines
docker exec -it <container> sh             --- Opens a shell inside a running container
docker exec <container> <command>          --- Runs a command inside a container
docker inspect <container>                 --- Shows detailed container info as JSON
docker stats                               --- Shows live CPU and memory usage
docker cp <container>:<path> <local-path>  --- Copies a file out of a container
```

#### Images & Registries

```text
# ---Build Images---
docker build -t <name> .                                            --- Builds an image from the current folder
docker build -t <name>:<tag> .                                      --- Builds an image with a tag
docker build -f <Dockerfile> -t <name> .                            --- Builds from a specific Dockerfile
docker build --no-cache -t <name> .                                 --- Builds without using the cache
docker build --build-arg <KEY>=<value> -t <name> .                  --- Passes a build argument
docker buildx build --platform linux/amd64,linux/arm64 -t <name> .  --- Builds for multiple CPU types

# ---Images & Tags---
docker images                           --- Lists local images
docker pull <image>                     --- Downloads an image
docker tag <image> <repo>/<name>:<tag>  --- Adds a new name to an image
docker push <repo>/<name>:<tag>         --- Uploads an image to a registry
docker rmi <image>                      --- Removes an image
docker history <image>                  --- Shows an image's layers
docker save -o <file.tar> <image>       --- Saves an image to a tar file
docker load -i <file.tar>               --- Loads an image from a tar file
```

#### Storage, Networks & Cleanup

```text
# ---Volumes---
docker volume ls              --- Lists volumes
docker volume create <name>   --- Creates a volume
docker volume inspect <name>  --- Shows volume details
docker volume rm <name>       --- Removes a volume
docker volume prune           --- Removes unused volumes

# ---Networks---
docker network ls                                --- Lists networks
docker network create <name>                     --- Creates a network
docker network connect <network> <container>     --- Connects a container to a network
docker network disconnect <network> <container>  --- Disconnects a container from a network
docker network inspect <name>                    --- Shows network details
docker run --network <name> <image>              --- Runs a container on a network

# ---Clean Up Space---
docker container prune              --- Removes all stopped containers
docker image prune                  --- Removes dangling images
docker image prune -a               --- Removes all unused images
docker system prune                 --- Removes unused containers, networks, and images
docker system prune -a --volumes    --- Removes everything unused, including volumes
[unix] docker stop $(docker ps -q)  --- Stops all running containers
[unix] docker rm $(docker ps -aq)   --- Removes all containers
```

### Docker Compose

**Flag:** `-compose`  |  **Also matches:** docker-compose  |  **Platforms:** Windows, Linux, macOS

#### Everything Compose

```text
# ---Start & Stop---
docker compose up                 --- Starts the services
docker compose up -d              --- Starts the services in the background
docker compose up --build         --- Rebuilds images, then starts the services
docker compose down               --- Stops and removes the containers
docker compose down -v            --- Also removes the volumes
docker compose stop               --- Stops the services without removing them
docker compose restart <service>  --- Restarts one service

# ---Build & Update---
docker compose build                    --- Builds the service images
docker compose build --no-cache         --- Builds without using the cache
docker compose pull                     --- Pulls the latest images
docker compose up -d --build <service>  --- Rebuilds and restarts one service
docker compose up -d --force-recreate   --- Recreates the containers

# ---Logs & Status---
docker compose ps                 --- Lists the project's containers
docker compose logs               --- Shows logs from all services
docker compose logs -f <service>  --- Follows one service's logs
docker compose top                --- Shows running processes
docker compose config             --- Validates and prints the final config
docker compose images             --- Lists the images in use

# ---Run Commands in Services---
docker compose exec <service> sh             --- Opens a shell in a running service
docker compose exec <service> <command>      --- Runs a command in a running service
docker compose run --rm <service> <command>  --- Runs a one-off command in a new container

# ---Files, Profiles & Scaling---
docker compose -f <file.yml> up             --- Uses a specific compose file
docker compose --env-file <file> up         --- Uses a specific env file
docker compose -p <project> up              --- Sets the project name
docker compose --profile <name> up          --- Starts services in a profile
docker compose up -d --scale <service>=<n>  --- Runs several copies of a service
```

### Kubernetes (kubectl)

**Flag:** `-kubectl`  |  **Also matches:** k8s, kubernetes, k  |  **Platforms:** Windows, Linux, macOS

#### Cluster & Resources

```text
# ---Contexts & Namespaces---
kubectl config get-contexts                            --- Lists your cluster contexts
kubectl config current-context                         --- Prints the active context
kubectl config use-context <context>                   --- Switches to another context
kubectl config set-context --current --namespace=<ns>  --- Sets the default namespace
kubectl get namespaces                                 --- Lists namespaces
kubectl create namespace <ns>                          --- Creates a namespace
kubectl cluster-info                                   --- Shows cluster endpoint details

# ---View Resources---
kubectl get pods                                          --- Lists pods in the current namespace
kubectl get pods -A                                       --- Lists pods in all namespaces
kubectl get pods -o wide                                  --- Lists pods with node and IP details
kubectl get all                                           --- Lists the main resources in a namespace
kubectl get deployments                                   --- Lists deployments
kubectl get services                                      --- Lists services
kubectl get nodes                                         --- Lists cluster nodes
kubectl get <resource> -o yaml                            --- Prints a resource as YAML
kubectl get pods -l <key>=<value>                         --- Filters pods by label
kubectl get events --sort-by=.metadata.creationTimestamp  --- Lists recent events in order

# ---Inspect & Debug---
kubectl describe pod <pod>                          --- Shows pod details and events
kubectl logs <pod>                                  --- Shows a pod's logs
kubectl logs -f <pod>                               --- Follows a pod's logs live
kubectl logs <pod> -c <container>                   --- Shows logs from one container
kubectl logs --previous <pod>                       --- Shows logs from the last crashed container
kubectl exec -it <pod> -- sh                        --- Opens a shell inside a pod
kubectl port-forward <pod> <local-port>:<pod-port>  --- Forwards a local port to a pod
kubectl top pods                                    --- Shows pod CPU and memory usage
kubectl top nodes                                   --- Shows node CPU and memory usage
```

#### Deploy & Configure

```text
# ---Apply & Delete---
kubectl apply -f <file.yaml>                      --- Creates or updates resources from a file
kubectl apply -f <folder>/                        --- Applies every manifest in a folder
kubectl diff -f <file.yaml>                       --- Previews changes before applying
kubectl delete -f <file.yaml>                     --- Deletes the resources in a file
kubectl delete pod <pod>                          --- Deletes a pod
kubectl create deployment <name> --image=<image>  --- Creates a deployment
kubectl expose deployment <name> --port=<port>    --- Creates a service for a deployment

# ---Deployments & Rollouts---
kubectl rollout status deployment/<name>                 --- Watches a rollout's progress
kubectl rollout history deployment/<name>                --- Lists a deployment's revisions
kubectl rollout undo deployment/<name>                   --- Rolls back to the previous revision
kubectl rollout restart deployment/<name>                --- Restarts a deployment's pods
kubectl scale deployment <name> --replicas=<n>           --- Sets the number of pods
kubectl set image deployment/<name> <container>=<image>  --- Updates a deployment's image

# ---ConfigMaps & Secrets---
kubectl create configmap <name> --from-literal=<KEY>=<value>       --- Creates a ConfigMap
kubectl create secret generic <name> --from-literal=<KEY>=<value>  --- Creates a secret
kubectl get secret <name> -o yaml                                  --- Shows a secret's encoded data
kubectl edit <resource> <name>                                     --- Edits a resource in your editor
```

### GitHub CLI (gh)

**Flag:** `-gh`  |  **Also matches:** github, github-cli, actions  |  **Platforms:** Windows, Linux, macOS

#### Everything gh

```text
# ---Log In & Set Up---
gh auth login                  --- Logs in to GitHub
gh auth status                 --- Shows your login status
gh auth logout                 --- Logs out of GitHub
gh config set editor <editor>  --- Sets the editor gh opens

# ---Repositories---
gh repo create <name> --public                     --- Creates a public repo on GitHub
gh repo create <name> --private --source=. --push  --- Creates a private repo from this folder and pushes
gh repo clone <owner>/<repo>                       --- Clones a repository
gh repo fork                                       --- Forks the current repository
gh repo view --web                                 --- Opens the repo in the browser
gh repo list                                       --- Lists your repositories

# ---Pull Requests---
gh pr create                   --- Creates a pull request interactively
gh pr create --fill            --- Creates a PR using the commit messages
gh pr list                     --- Lists open pull requests
gh pr view <number>            --- Shows a pull request
gh pr checkout <number>        --- Checks out a pull request locally
gh pr diff                     --- Shows the PR's changes
gh pr merge <number> --squash  --- Squash-merges a pull request
gh pr status                   --- Shows PRs that involve you

# ---Issues---
gh issue create                            --- Creates an issue interactively
gh issue list                              --- Lists open issues
gh issue view <number>                     --- Shows an issue
gh issue close <number>                    --- Closes an issue
gh issue comment <number> --body "<text>"  --- Adds a comment to an issue

# ---Releases---
gh release create <tag> --generate-notes  --- Creates a release with generated notes
gh release create <tag> <files>           --- Creates a release with attached files
gh release list                           --- Lists releases
gh release download <tag>                 --- Downloads a release's assets
gh release delete <tag>                   --- Deletes a release

# ---Workflow Runs (Actions)---
gh run list                    --- Lists recent workflow runs
gh run watch                   --- Watches a run until it finishes
gh run view <id> --log-failed  --- Shows logs from failed steps
gh run rerun <id>              --- Reruns a workflow run
gh workflow list               --- Lists workflows
gh workflow run <workflow>     --- Triggers a workflow manually

# ---Secrets & API---
gh secret set <NAME>   --- Sets a repository secret
gh secret list         --- Lists repository secrets
gh api <endpoint>      --- Calls the GitHub API
gh gist create <file>  --- Creates a gist from a file
```

### SSH, SCP & rsync

**Flag:** `-ssh`  |  **Also matches:** scp, rsync, remote-server  |  **Platforms:** Windows, Linux, macOS

> **Note:** rsync is not installed on Windows by default. Use WSL or Git Bash there

#### Remote Access

```text
# ---Connect to a Server---
ssh <user>@<host>                --- Connects to a server
ssh -p <port> <user>@<host>      --- Connects on a custom port
ssh -i <key-file> <user>@<host>  --- Connects using a specific key
ssh <user>@<host> "<command>"    --- Runs one command and disconnects
ssh -v <user>@<host>             --- Shows detailed output for debugging

# ---SSH Keys---
ssh-keygen -t ed25519 -C "<email>"            --- Creates a new Ed25519 key pair
ssh-keygen -t rsa -b 4096 -C "<email>"        --- Creates a 4096-bit RSA key pair
[unix] ssh-copy-id <user>@<host>              --- Installs your public key on a server
[unix] cat ~/.ssh/id_ed25519.pub              --- Prints your public key
[win] type %USERPROFILE%\.ssh\id_ed25519.pub  --- Prints your public key
ssh -T git@github.com                         --- Tests your GitHub SSH connection
ssh-keygen -R <host>                          --- Removes a host from known_hosts
[unix] chmod 600 ~/.ssh/id_ed25519            --- Fixes private key permissions

# ---SSH Agent---
[unix] eval "$(ssh-agent -s)"  --- Starts the SSH agent
ssh-add <key-file>             --- Adds a key to the agent
ssh-add -l                     --- Lists keys in the agent
ssh-add -D                     --- Removes all keys from the agent
```

#### Copying & Tunnels

```text
# ---Copy Files (scp)---
scp <file> <user>@<host>:<remote-path>                --- Uploads a file to a server
scp <user>@<host>:<remote-file> <local-path>          --- Downloads a file from a server
scp -r <folder> <user>@<host>:<remote-path>           --- Uploads a folder
scp -P <port> <file> <user>@<host>:<remote-path>      --- Uploads using a custom port
scp -i <key-file> <file> <user>@<host>:<remote-path>  --- Uploads using a specific key

# ---Sync Folders (rsync)---
rsync -avz <source>/ <user>@<host>:<dest>/                --- Syncs a folder to a server
rsync -avz --progress <source>/ <dest>/                   --- Syncs and shows progress
rsync -avz --delete <source>/ <dest>/                     --- Syncs and deletes extra files at the destination
rsync -avzn <source>/ <dest>/                             --- Previews a sync without changing anything
rsync -avz --exclude '<pattern>' <source>/ <dest>/        --- Syncs while skipping matching files
rsync -e "ssh -p <port>" <source>/ <user>@<host>:<dest>/  --- Syncs over a custom SSH port

# ---Tunnels & Jump Hosts---
ssh -L <local-port>:localhost:<remote-port> <user>@<host>        --- Forwards a local port to the server
ssh -R <remote-port>:localhost:<local-port> <user>@<host>        --- Exposes a local port on the server
ssh -D <port> <user>@<host>                                      --- Starts a SOCKS proxy through the server
ssh -N -f -L <local-port>:localhost:<remote-port> <user>@<host>  --- Runs a tunnel in the background
ssh -J <jump-host> <user>@<host>                                 --- Connects through a jump host
```

### Terraform

**Flag:** `-terraform`  |  **Also matches:** tf, iac  |  **Platforms:** Windows, Linux, macOS

#### Everything Terraform

```text
# ---Set Up & Format---
terraform version        --- Prints the Terraform version
terraform init           --- Downloads providers and sets up the backend
terraform init -upgrade  --- Updates providers to the newest allowed versions
terraform fmt            --- Formats the configuration files
terraform validate       --- Checks the configuration for errors

# ---Plan & Apply---
terraform plan                           --- Previews the changes
terraform plan -out=<file>               --- Saves the plan to a file
terraform apply                          --- Applies the changes
terraform apply <file>                   --- Applies a saved plan
terraform apply -auto-approve            --- Applies without a confirmation prompt
terraform apply -var-file=<file.tfvars>  --- Applies using a variables file
terraform apply -replace=<resource>      --- Recreates a single resource
terraform destroy                        --- Destroys all managed resources

# ---State & Imports---
terraform state list              --- Lists resources in the state
terraform state show <resource>   --- Shows one resource's details
terraform state rm <resource>     --- Removes a resource from the state
terraform import <resource> <id>  --- Imports an existing resource
terraform output                  --- Prints the output values
terraform show                    --- Shows the current state or a plan

# ---Workspaces---
terraform workspace list           --- Lists workspaces
terraform workspace new <name>     --- Creates a workspace
terraform workspace select <name>  --- Switches to a workspace
terraform workspace show           --- Prints the current workspace
terraform workspace delete <name>  --- Deletes a workspace

# ---Debugging---
terraform graph                     --- Prints a dependency graph
terraform providers                 --- Lists the providers in use
terraform force-unlock <lock-id>    --- Releases a stuck state lock
[unix] TF_LOG=DEBUG terraform plan  --- Runs a plan with debug logging
[win] set TF_LOG=DEBUG              --- Turns on debug logging for this terminal
```

### Nginx & Certbot

**Flag:** `-nginx`  |  **Also matches:** certbot, reverse-proxy, https  |  **Platforms:** Linux, macOS

> **Note:** Linux commands use systemd and Debian-style paths. On macOS, Nginx comes from Homebrew, so services and file paths differ

#### Everything Nginx

```text
# ---Test & Reload---
sudo nginx -t                  --- Tests the configuration for errors
sudo nginx -s reload           --- Reloads the config without downtime
[linux] sudo systemctl restart nginx  --- Restarts Nginx
[linux] sudo systemctl status nginx   --- Shows whether Nginx is running
[mac] brew services restart nginx     --- Restarts Nginx
[mac] brew services info nginx        --- Shows whether Nginx is running
nginx -V                       --- Prints version and build options

# ---Sites & Logs---
[linux] sudo ln -s /etc/nginx/sites-available/<site> /etc/nginx/sites-enabled/  --- Enables a site (Debian and Ubuntu)
[linux] sudo rm /etc/nginx/sites-enabled/<site>                                 --- Disables a site (Debian and Ubuntu)
[linux] sudo tail -f /var/log/nginx/error.log                                   --- Follows the error log
[linux] sudo tail -f /var/log/nginx/access.log                                  --- Follows the access log
[mac] tail -f $(brew --prefix)/var/log/nginx/error.log                          --- Follows the error log
[mac] tail -f $(brew --prefix)/var/log/nginx/access.log                         --- Follows the access log
[mac] ls $(brew --prefix)/etc/nginx/servers                                     --- Lists the site configs Nginx loads

# ---HTTPS with Certbot---
[mac] brew install certbot                --- Installs Certbot
sudo certbot --nginx -d <domain>          --- Gets a certificate and configures Nginx
sudo certbot certificates                 --- Lists your certificates
sudo certbot renew --dry-run              --- Tests automatic renewal
sudo certbot renew                        --- Renews certificates that are close to expiring
sudo certbot delete --cert-name <domain>  --- Deletes a certificate
```
---

## Cloud & Backend Services

### Firebase CLI

**Flag:** `-firebase`  |  **Also matches:** firebase-tools, flutterfire  |  **Platforms:** Windows, Linux, macOS

#### Everything Firebase

```text
# ---Log In & Set Up---
npm install -g firebase-tools  --- Installs the Firebase CLI
firebase login                 --- Logs in to Firebase
firebase logout                --- Logs out of Firebase
firebase projects:list         --- Lists your projects
firebase use <project-id>      --- Selects the active project
firebase init                  --- Sets up Firebase in the current folder

# ---Emulators---
firebase init emulators                                   --- Sets up the local emulators
firebase emulators:start                                  --- Starts all configured emulators
firebase emulators:start --only <emulators>               --- Starts only some emulators
firebase emulators:start --import=<dir>                   --- Starts with saved emulator data
firebase emulators:start --import=<dir> --export-on-exit  --- Saves emulator data when you stop
firebase emulators:export <dir>                           --- Saves the emulator data

# ---Deploy---
firebase deploy                            --- Deploys everything configured
firebase deploy --only hosting             --- Deploys only hosting
firebase deploy --only functions           --- Deploys only Cloud Functions
firebase deploy --only functions:<name>    --- Deploys one function
firebase deploy --only firestore:rules     --- Deploys only Firestore rules
firebase hosting:channel:deploy <channel>  --- Deploys a preview channel

# ---Cloud Functions---
firebase init functions                --- Sets up Cloud Functions
firebase functions:list                --- Lists deployed functions
firebase functions:log                 --- Shows function logs
firebase functions:delete <name>       --- Deletes a deployed function
firebase functions:secrets:set <NAME>  --- Stores a secret for functions

# ---FlutterFire Setup---
dart pub global activate flutterfire_cli      --- Installs the FlutterFire CLI
flutterfire configure                         --- Connects a Flutter app to Firebase
flutterfire configure --project=<project-id>  --- Connects the app to a specific project
```

### Supabase CLI

**Flag:** `-supabase`  |  **Also matches:** supabase-cli, postgres-backend  |  **Platforms:** Windows, Linux, macOS

#### Everything Supabase

```text
# ---Set Up & Link---
npx supabase login                     --- Logs in to Supabase
npx supabase init                      --- Creates a supabase folder in the project
npx supabase projects list             --- Lists your projects
npx supabase link --project-ref <ref>  --- Links the folder to a hosted project

# ---Local Development---
npx supabase start     --- Starts the local stack in Docker
npx supabase stop      --- Stops the local stack
npx supabase status    --- Shows local URLs and keys
npx supabase db reset  --- Rebuilds the local database from migrations

# ---Migrations---
npx supabase migration new <name>            --- Creates a new migration file
npx supabase migration list                  --- Lists applied migrations
npx supabase db diff -f <name>               --- Creates a migration from schema changes
npx supabase db push                         --- Applies migrations to the linked project
npx supabase db pull                         --- Pulls the remote schema as a migration
npx supabase db dump --linked -f <file.sql>  --- Exports the linked project's schema

# ---Generate Types---
npx supabase gen types --lang typescript --local > <types.ts>             --- Generates types from the local database
npx supabase gen types --lang typescript --linked > <types.ts>            --- Generates types from the linked project
npx supabase gen types --lang typescript --project-id <ref> > <types.ts>  --- Generates types from any project by ID

# ---Edge Functions & Secrets---
npx supabase functions new <name>       --- Creates an edge function
npx supabase functions serve            --- Runs edge functions locally
npx supabase functions deploy <name>    --- Deploys an edge function
npx supabase functions list             --- Lists deployed functions
npx supabase secrets set <KEY>=<value>  --- Sets a secret for functions
npx supabase secrets list               --- Lists your secrets
npx supabase secrets unset <KEY>        --- Removes a secret
```

### AWS CLI

**Flag:** `-aws`  |  **Also matches:** amazon, s3, ec2, lambda  |  **Platforms:** Windows, Linux, macOS

#### Everything AWS

```text
# ---Log In & Profiles---
aws configure                   --- Sets up access keys and a default region
aws configure --profile <name>  --- Sets up a named profile
aws configure sso               --- Sets up single sign-on
aws sso login --profile <name>  --- Logs in with single sign-on
aws sts get-caller-identity     --- Shows which account you are using
aws configure list-profiles     --- Lists your profiles
aws --profile <name> <command>  --- Runs a command with a profile

# ---S3 Storage---
aws s3 ls                             --- Lists your buckets
aws s3 ls s3://<bucket>               --- Lists the contents of a bucket
aws s3 cp <file> s3://<bucket>/<key>  --- Uploads a file
aws s3 cp s3://<bucket>/<key> <file>  --- Downloads a file
aws s3 sync <folder> s3://<bucket>    --- Syncs a folder to a bucket
aws s3 rm s3://<bucket>/<key>         --- Deletes an object
aws s3 rm s3://<bucket> --recursive   --- Deletes everything in a bucket
aws s3 presign s3://<bucket>/<key>    --- Creates a temporary download link

# ---EC2 Servers---
aws ec2 describe-instances                       --- Lists your EC2 instances
aws ec2 start-instances --instance-ids <id>      --- Starts an instance
aws ec2 stop-instances --instance-ids <id>       --- Stops an instance
aws ec2 reboot-instances --instance-ids <id>     --- Reboots an instance
aws ec2 terminate-instances --instance-ids <id>  --- Permanently deletes an instance

# ---Lambda & Logs---
aws lambda list-functions                            --- Lists your Lambda functions
aws lambda invoke --function-name <name> <out.json>  --- Runs a function and saves the result
aws logs describe-log-groups                         --- Lists CloudWatch log groups
aws logs tail <log-group> --follow                   --- Follows a log group live
```

### Google Cloud CLI (gcloud)

**Flag:** `-gcloud`  |  **Also matches:** google-cloud, gcp, cloud-run  |  **Platforms:** Windows, Linux, macOS

#### Everything Google Cloud

```text
# ---Log In & Configure---
gcloud init                             --- Sets up gcloud interactively
gcloud auth login                       --- Logs in with your Google account
gcloud auth list                        --- Lists logged-in accounts
gcloud auth application-default login   --- Creates credentials for local code
gcloud config list                      --- Shows the current configuration
gcloud config set project <project-id>  --- Sets the active project
gcloud projects list                    --- Lists your projects

# ---Compute & Cloud Run---
gcloud compute instances list           --- Lists virtual machines
gcloud compute ssh <instance>           --- Connects to a VM over SSH
gcloud run deploy <service> --source .  --- Builds and deploys to Cloud Run
gcloud run services list                --- Lists Cloud Run services
gcloud app deploy                       --- Deploys to App Engine

# ---Cloud Storage---
gcloud storage ls                                --- Lists your buckets
gcloud storage ls gs://<bucket>                  --- Lists the contents of a bucket
gcloud storage cp <file> gs://<bucket>           --- Uploads a file
gcloud storage cp gs://<bucket>/<object> <file>  --- Downloads a file
gcloud storage rsync <folder> gs://<bucket>      --- Syncs a folder to a bucket
```

### Azure CLI

**Flag:** `-az`  |  **Also matches:** azure, microsoft-azure  |  **Platforms:** Windows, Linux, macOS

#### Everything Azure

```text
# ---Log In & Subscriptions---
az login                            --- Logs in to Azure
az account list                     --- Lists your subscriptions
az account set --subscription <id>  --- Sets the active subscription
az account show                     --- Shows the active subscription
az logout                           --- Logs out of Azure

# ---Resource Groups---
az group list                                      --- Lists resource groups
az group create --name <name> --location <region>  --- Creates a resource group
az group delete --name <name>                      --- Deletes a resource group and its contents
az resource list                                   --- Lists all resources

# ---Web Apps & VMs---
az webapp list                                      --- Lists web apps
az webapp up --name <name>                          --- Deploys the current folder as a web app
az vm list                                          --- Lists virtual machines
az vm start --name <name> --resource-group <group>  --- Starts a virtual machine
az vm stop --name <name> --resource-group <group>   --- Stops a virtual machine
```

---

## Languages & Package Tools

### Python & pip

**Flag:** `-python`  |  **Also matches:** pip, venv, virtualenv, pytest  |  **Platforms:** Windows, Linux, macOS

> **Note:** Windows usually uses python or the py launcher, while Linux and macOS use python3

#### Setup & Environments

```text
# ---Check Your Setup---
[unix] python3 --version  --- Prints the Python version
[win] py --version        --- Prints the Python version
pip --version             --- Prints the pip version and its Python
[unix] which python3      --- Shows where Python is installed
[win] where python        --- Shows where Python is installed

# ---Virtual Environments---
[unix] python3 -m venv .venv      --- Creates a virtual environment
[win] python -m venv .venv        --- Creates a virtual environment
[unix] source .venv/bin/activate  --- Activates the environment
[win] .venv\Scripts\activate      --- Activates the environment in CMD
[win] .venv\Scripts\Activate.ps1  --- Activates the environment in PowerShell
deactivate                        --- Leaves the environment
[unix] rm -rf .venv               --- Deletes the environment
[win] rmdir /s /q .venv           --- Deletes the environment
```

#### Packages

```text
# ---Install Packages---
pip install <pkg>                    --- Installs a package
pip install <pkg>==<version>         --- Installs a specific version
pip install -r requirements.txt      --- Installs everything in a requirements file
pip install --upgrade <pkg>          --- Upgrades a package
pip install -e .                     --- Installs the current project in editable mode
pip uninstall <pkg>                  --- Removes a package
python -m pip install --upgrade pip  --- Upgrades pip itself

# ---Inspect & Export Packages---
pip list                       --- Lists installed packages
pip list --outdated            --- Lists packages with newer versions
pip show <pkg>                 --- Shows a package's details
pip freeze                     --- Prints installed packages with versions
pip freeze > requirements.txt  --- Saves installed packages to a file
pip check                      --- Checks for broken dependencies
pip cache purge                --- Clears the pip cache
```

#### Running & Quality

```text
# ---Run Code & Quick Servers---
python <file.py>                 --- Runs a Python file
python -c "<code>"               --- Runs a one-line script
python -m http.server 8000       --- Serves the current folder over HTTP
python -m json.tool <file.json>  --- Pretty-prints a JSON file
python -m pdb <file.py>          --- Runs a file in the debugger
python -i <file.py>              --- Runs a file, then opens an interactive prompt

# ---Test, Lint & Format---
python -m pytest                    --- Runs all tests
python -m pytest -v                 --- Runs tests with detailed output
python -m pytest <file>::<test>     --- Runs one test
python -m pytest -k "<expression>"  --- Runs tests matching a name
python -m pytest -x                 --- Stops at the first failure
ruff check .                        --- Lints the code
ruff format .                       --- Formats the code
mypy .                              --- Type-checks the code

# ---Build & Publish---
pip install build twine                    --- Installs the packaging tools
python -m build                            --- Builds the package files
twine check dist/*                         --- Validates the built files
twine upload --repository testpypi dist/*  --- Uploads to TestPyPI
twine upload dist/*                        --- Uploads to PyPI
```

### Django, Flask & FastAPI

**Flag:** `-pyweb`  |  **Also matches:** django, flask, fastapi, uvicorn  |  **Platforms:** Windows, Linux, macOS

#### Python Web Frameworks

```text
# ---Django---
django-admin startproject <name>  --- Creates a new Django project
python manage.py startapp <app>   --- Creates a new app in the project
python manage.py runserver        --- Starts the dev server
python manage.py makemigrations   --- Creates migrations from model changes
python manage.py migrate          --- Applies migrations
python manage.py createsuperuser  --- Creates an admin user
python manage.py collectstatic    --- Gathers static files for deployment
python manage.py shell            --- Opens a Python shell with Django loaded

# ---Flask---
pip install flask                 --- Installs Flask
flask run                         --- Runs the Flask dev server
flask --app <module> run --debug  --- Runs with debug mode and reloading
flask --app <module> routes       --- Lists the app's routes
flask --app <module> shell        --- Opens a shell with the app context

# ---FastAPI---
pip install "fastapi[standard]"                    --- Installs FastAPI with its standard extras
fastapi dev <main.py>                              --- Runs the dev server with reloading
fastapi run <main.py>                              --- Runs the production server
uvicorn <module>:app --reload                      --- Runs the app with Uvicorn and reloading
uvicorn <module>:app --host 0.0.0.0 --port <port>  --- Serves the app on your network
```

### uv

**Flag:** `-uv`  |  **Also matches:** uvx, astral  |  **Platforms:** Windows, Linux, macOS

#### Everything uv

```text
# ---Projects & Dependencies---
uv init <name>                   --- Creates a new Python project
uv add <pkg>                     --- Adds a dependency
uv add --dev <pkg>               --- Adds a dev dependency
uv remove <pkg>                  --- Removes a dependency
uv sync                          --- Installs dependencies from the lockfile
uv lock                          --- Updates the lockfile
uv lock --upgrade                --- Upgrades all dependencies in the lockfile
uv lock --upgrade-package <pkg>  --- Upgrades one dependency in the lockfile
uv run <file.py>                 --- Runs a file in the project environment
uv run <command>                 --- Runs a command in the project environment
uv tree                          --- Prints the dependency tree

# ---Python Versions & Environments---
uv python list               --- Lists available Python versions
uv python install <version>  --- Installs a Python version
uv python pin <version>      --- Pins the project's Python version
uv venv                      --- Creates a virtual environment
uv venv --python <version>   --- Creates one with a specific Python version

# ---pip Interface---
uv pip install <pkg>                                --- Installs a package
uv pip install -r requirements.txt                  --- Installs from a requirements file
uv pip freeze                                       --- Lists installed packages
uv pip compile requirements.in -o requirements.txt  --- Locks requirements with exact versions

# ---Tools & Maintenance---
uvx <tool>              --- Runs a tool without installing it
uv tool install <tool>  --- Installs a command-line tool
uv tool list            --- Lists installed tools
uv tool upgrade --all   --- Upgrades all installed tools
uv cache clean          --- Clears the uv cache
uv self update          --- Updates uv itself
```

### Poetry

**Flag:** `-poetry`  |  **Also matches:** pyproject  |  **Platforms:** Windows, Linux, macOS

#### Everything Poetry

```text
# ---Projects & Dependencies---
poetry new <name>             --- Creates a new project
poetry init                   --- Creates a pyproject.toml interactively
poetry add <pkg>              --- Adds a dependency
poetry add --group dev <pkg>  --- Adds a dev dependency
poetry remove <pkg>           --- Removes a dependency
poetry install                --- Installs dependencies from the lockfile
poetry update                 --- Updates dependencies and the lockfile
poetry lock                   --- Updates the lockfile only
poetry show --tree            --- Prints the dependency tree

# ---Environments & Running---
poetry run <command>                       --- Runs a command in the project environment
poetry run python <file.py>                --- Runs a Python file in the environment
poetry env info                            --- Shows the active environment
poetry env list                            --- Lists the project's environments
poetry env activate                        --- Prints the command that activates the environment
[unix] eval $(poetry env activate)         --- Activates the environment in this shell
poetry env use <python>                    --- Chooses which Python the project uses
poetry self add poetry-plugin-shell        --- Brings back the poetry shell command (Poetry 2+)
poetry env remove <python>                 --- Deletes an environment
poetry config virtualenvs.in-project true  --- Creates the environment inside the project

# ---Build & Publish---
poetry build            --- Builds the package files
poetry publish          --- Uploads the package to PyPI
poetry publish --build  --- Builds, then uploads the package
poetry version patch    --- Bumps the patch version
```

### Conda

**Flag:** `-conda`  |  **Also matches:** anaconda, miniconda, mamba  |  **Platforms:** Windows, Linux, macOS

#### Everything Conda

```text
# ---Environments---
conda create -n <env> python=<version>  --- Creates a new environment
conda activate <env>                    --- Activates an environment
conda deactivate                        --- Leaves the current environment
conda env list                          --- Lists all environments
conda remove -n <env> --all             --- Deletes an environment

# ---Packages---
conda install <pkg>                 --- Installs a package
conda install -c conda-forge <pkg>  --- Installs from the conda-forge channel
conda list                          --- Lists installed packages
conda remove <pkg>                  --- Removes a package
conda update <pkg>                  --- Updates a package
conda search <pkg>                  --- Searches for a package

# ---Export, Share & Clean Up---
conda env export > environment.yml           --- Saves the environment to a file
conda env create -f environment.yml          --- Creates an environment from a file
conda update conda                           --- Updates conda itself
conda clean --all                            --- Clears unused packages and caches
conda config --set auto_activate_base false  --- Stops activating base automatically
conda init <shell>                           --- Sets up conda for your shell
```

### Rust & Cargo

**Flag:** `-cargo`  |  **Also matches:** rust, rustup, rustc, crate  |  **Platforms:** Windows, Linux, macOS

#### Projects & Toolchain

```text
# ---Create a Project---
cargo new <name>        --- Creates a new binary project
cargo new <name> --lib  --- Creates a new library project
cargo init              --- Sets up Cargo in the current folder
rustc --version         --- Prints the Rust compiler version
cargo --version         --- Prints the Cargo version

# ---Toolchains & Targets (rustup)---
rustup update                        --- Updates Rust to the latest version
rustup show                          --- Shows the active toolchain and targets
rustup default stable                --- Sets the default toolchain
rustup toolchain install nightly     --- Installs the nightly toolchain
rustup target add <triple>           --- Adds a compile target
rustup component add clippy rustfmt  --- Installs the lint and format tools
rustup override set <toolchain>      --- Sets the toolchain for this folder
cargo +nightly <command>             --- Runs a command with the nightly toolchain
```

#### Build & Dependencies

```text
# ---Build & Run---
cargo build                    --- Compiles in debug mode
cargo build --release          --- Compiles with optimizations
cargo run                      --- Builds and runs the program
cargo run --release            --- Builds and runs with optimizations
cargo run -- <args>            --- Runs the program with arguments
cargo run --bin <name>         --- Runs a specific binary
cargo check                    --- Checks the code without building
cargo build --target <triple>  --- Compiles for another target
cargo clean                    --- Deletes the target folder

# ---Dependencies---
cargo add <crate>                    --- Adds a dependency
cargo add <crate> --features <list>  --- Adds a dependency with features
cargo add --dev <crate>              --- Adds a dev dependency
cargo remove <crate>                 --- Removes a dependency
cargo update                         --- Updates dependencies in Cargo.lock
cargo update -p <crate>              --- Updates a single crate
cargo tree                           --- Prints the dependency tree
cargo tree -d                        --- Shows duplicate dependencies
```

#### Quality & Publishing

```text
# ---Test & Benchmark---
cargo test                 --- Runs all tests
cargo test <name>          --- Runs tests matching a name
cargo test -- --nocapture  --- Shows println output from tests
cargo test --lib           --- Runs only library tests
cargo test --doc           --- Runs only documentation tests
cargo bench                --- Runs benchmarks

# ---Lint & Format---
cargo fmt                    --- Formats the code
cargo fmt --check            --- Checks formatting without changing files
cargo clippy                 --- Runs the linter
cargo clippy -- -D warnings  --- Treats lint warnings as errors
cargo clippy --fix           --- Applies lint fixes automatically
cargo doc --open             --- Builds the docs and opens them

# ---Install Tools---
cargo install <crate>          --- Installs a command-line tool
cargo install --path .         --- Installs the current project as a tool
cargo install --list           --- Lists installed tools
cargo install --force <crate>  --- Reinstalls or updates a tool
cargo uninstall <crate>        --- Removes an installed tool

# ---Publish a Crate---
cargo login                     --- Logs in to crates.io
cargo package                   --- Creates the package that would be published
cargo publish --dry-run         --- Previews a publish
cargo publish                   --- Publishes the crate to crates.io
cargo yank --version <version>  --- Marks a version as not recommended
```

### Go

**Flag:** `-go`  |  **Also matches:** golang, go-modules  |  **Platforms:** Windows, Linux, macOS

#### Everything Go

```text
# ---Modules & Setup---
go version            --- Prints the Go version
go env                --- Prints the Go environment settings
go mod init <module>  --- Creates a new module
go mod tidy           --- Adds missing and removes unused dependencies
go mod download       --- Downloads the module's dependencies
go mod vendor         --- Copies dependencies into a vendor folder
go mod why <pkg>      --- Explains why a package is needed

# ---Run & Build---
go run .                 --- Runs the program in this folder
go run <file.go>         --- Runs a Go file
go build                 --- Compiles the program
go build -o <name>       --- Compiles to a named file
go build ./...           --- Builds every package
go install <pkg>@latest  --- Installs a command-line tool

# ---Cross-Compile---
[unix] GOOS=linux GOARCH=amd64 go build    --- Builds for 64-bit Linux
[unix] GOOS=windows GOARCH=amd64 go build  --- Builds for 64-bit Windows
[unix] GOOS=darwin GOARCH=arm64 go build   --- Builds for Apple Silicon
[win] set GOOS=linux                       --- Sets the target OS for this terminal
[win] set GOARCH=amd64                     --- Sets the target CPU for this terminal
go tool dist list                          --- Lists every supported OS and CPU pair

# ---Dependencies---
go get <pkg>                      --- Adds a dependency
go get <pkg>@<version>            --- Adds a specific version
go get -u <pkg>                   --- Updates a dependency
go get -u ./...                   --- Updates all dependencies
go list -m all                    --- Lists all modules in use
go list -m -u all                 --- Lists modules with updates available
go mod edit -replace=<old>=<new>  --- Replaces a module with another path

# ---Test---
go test                 --- Runs tests in this package
go test ./...           --- Runs tests in every package
go test -v ./...        --- Runs tests with detailed output
go test -run <name>     --- Runs tests matching a name
go test -cover ./...    --- Shows test coverage
go test -race ./...     --- Detects data races
go test -count=1 ./...  --- Runs tests without using the cache
go test -bench .        --- Runs benchmarks

# ---Format, Vet & Clean---
go fmt ./...        --- Formats the code
go vet ./...        --- Reports suspicious code
go doc <pkg>        --- Shows documentation for a package
go clean -cache     --- Clears the build cache
go clean -modcache  --- Clears the module cache
```

### Java

**Flag:** `-java`  |  **Also matches:** javac, jar, jshell, jdk  |  **Platforms:** Windows, Linux, macOS

#### Everything Java

```text
# ---Compile & Run---
java -version        --- Prints the Java runtime version
javac -version       --- Prints the compiler version
javac <File.java>    --- Compiles a Java file
java <ClassName>     --- Runs a compiled class
java <File.java>     --- Compiles and runs a single source file
java -jar <app.jar>  --- Runs a JAR file
jshell               --- Opens an interactive Java prompt

# ---Packaging & Classpath---
jar cf <app.jar> <files>          --- Creates a JAR file
jar tf <app.jar>                  --- Lists the contents of a JAR
java -cp <classpath> <ClassName>  --- Runs a class with a classpath
javac -d <out-dir> <files>        --- Compiles into an output folder
java -Xmx<size> -jar <app.jar>    --- Runs with a maximum memory size
```

### Maven

**Flag:** `-maven`  |  **Also matches:** mvn, pom  |  **Platforms:** Windows, Linux, macOS

#### Everything Maven

```text
# ---Build Lifecycle---
mvn -v                         --- Prints the Maven version
mvn clean                      --- Deletes the target folder
mvn compile                    --- Compiles the source code
mvn test                       --- Runs the tests
mvn package                    --- Builds the JAR or WAR file
mvn clean install              --- Cleans, builds, and installs to the local repository
mvn clean package -DskipTests  --- Builds without running tests
mvn -U clean install           --- Forces updates of snapshot dependencies

# ---Dependencies & Plugins---
mvn dependency:tree                      --- Prints the dependency tree
mvn dependency:resolve                   --- Downloads all dependencies
mvn versions:display-dependency-updates  --- Lists dependencies with newer versions
mvn archetype:generate                   --- Creates a project from a template
mvn test -Dtest=<ClassName>              --- Runs tests in one class
mvn spring-boot:run                      --- Runs a Spring Boot app
```

---

## Databases

### PostgreSQL (psql)

**Flag:** `-postgres`  |  **Also matches:** psql, postgresql, pg_dump  |  **Platforms:** Windows, Linux, macOS

#### Connect & Query

```text
# ---Connect---
psql -U <user> -d <database>                      --- Connects to a database
psql -h <host> -p <port> -U <user> -d <database>  --- Connects to a remote server
psql "<connection-string>"                        --- Connects using a connection string
psql -c "<sql>"                                   --- Runs one SQL command and exits
psql -f <file.sql>                                --- Runs the SQL in a file
pg_isready                                        --- Checks whether the server is accepting connections

# ---Inside psql---
\l             --- Lists databases
\c <database>  --- Switches to a database
\dt            --- Lists tables
\d <table>     --- Describes a table
\dn            --- Lists schemas
\du            --- Lists users and roles
\x             --- Toggles expanded output
\timing        --- Toggles query timing
\q             --- Quits psql
```

#### Admin & Backup

```text
# ---Users & Databases---
CREATE DATABASE <name>;                           --- Creates a database
CREATE USER <user> WITH PASSWORD '<password>';    --- Creates a user
GRANT ALL PRIVILEGES ON DATABASE <db> TO <user>;  --- Gives a user full access to a database
ALTER USER <user> WITH PASSWORD '<password>';     --- Changes a user's password
ALTER DATABASE <db> OWNER TO <user>;              --- Changes a database's owner
DROP DATABASE <name>;                             --- Deletes a database

# ---Command-Line Tools---
createdb <name>           --- Creates a database from the shell
dropdb <name>             --- Deletes a database from the shell
createuser --interactive  --- Creates a user with prompts
psql --version            --- Prints the psql version

# ---Backup & Restore---
pg_dump -U <user> <database> > <backup.sql>        --- Exports a database to a SQL file
pg_dump -Fc -U <user> <database> -f <backup.dump>  --- Exports in compressed custom format
psql -U <user> <database> < <backup.sql>           --- Restores from a SQL file
pg_restore -U <user> -d <database> <backup.dump>   --- Restores from a custom-format dump
pg_dumpall > <all.sql>                             --- Exports every database
```

### MySQL & MariaDB

**Flag:** `-mysql`  |  **Also matches:** mariadb, mysqldump  |  **Platforms:** Windows, Linux, macOS

#### Everything MySQL

```text
# ---Connect---
mysql -u <user> -p                      --- Connects to the server
mysql -u <user> -p <database>           --- Connects to a database
mysql -h <host> -P <port> -u <user> -p  --- Connects to a remote server
mysql -u <user> -p -e "<sql>"           --- Runs one SQL command and exits

# ---Inside mysql---
SHOW DATABASES;             --- Lists databases
USE <database>;             --- Switches to a database
SHOW TABLES;                --- Lists tables
DESCRIBE <table>;           --- Describes a table
SHOW CREATE TABLE <table>;  --- Shows the SQL that creates a table
EXIT;                       --- Quits the mysql client

# ---Users & Databases---
CREATE DATABASE <name>;                                       --- Creates a database
CREATE USER '<user>'@'localhost' IDENTIFIED BY '<password>';  --- Creates a user
GRANT ALL PRIVILEGES ON <db>.* TO '<user>'@'localhost';       --- Gives a user full access to a database
FLUSH PRIVILEGES;                                             --- Reloads the privilege tables
SHOW GRANTS FOR '<user>'@'localhost';                         --- Shows a user's permissions
DROP DATABASE <name>;                                         --- Deletes a database

# ---Backup & Restore---
mysqldump -u <user> -p <database> > <backup.sql>    --- Exports a database to a SQL file
mysqldump -u <user> -p --all-databases > <all.sql>  --- Exports every database
mysql -u <user> -p <database> < <backup.sql>        --- Restores from a SQL file
```

### SQLite

**Flag:** `-sqlite`  |  **Also matches:** sqlite3  |  **Platforms:** Windows, Linux, macOS

#### Everything SQLite

```text
# ---Open & Query---
sqlite3 <file.db>          --- Opens or creates a database
sqlite3 <file.db> "<sql>"  --- Runs one SQL command and exits
.read <file.sql>           --- Runs the SQL in a file
.quit                      --- Quits the sqlite3 shell

# ---Inspect (Dot Commands)---
.tables          --- Lists tables
.schema <table>  --- Shows the SQL that creates a table
.databases       --- Lists attached databases
.headers on      --- Shows column names in results
.mode column     --- Prints results in aligned columns

# ---Import & Export---
.mode csv                   --- Switches to CSV format
.import <file.csv> <table>  --- Imports a CSV file into a table
.output <file>              --- Sends results to a file
.output stdout              --- Sends results back to the screen

# ---Backup---
.dump                                   --- Prints the whole database as SQL
sqlite3 <file.db> .dump > <backup.sql>  --- Exports a database to a SQL file
.backup <file.db>                       --- Copies the database to another file
```

### MongoDB

**Flag:** `-mongodb`  |  **Also matches:** mongo, mongosh, mongodump  |  **Platforms:** Windows, Linux, macOS

#### Everything MongoDB

```text
# ---Connect & Browse---
mongosh                        --- Connects to a local server
mongosh "<connection-string>"  --- Connects using a connection string
show dbs                       --- Lists databases
use <database>                 --- Switches to a database
show collections               --- Lists collections

# ---Query & Edit Documents---
db.<collection>.find()                                             --- Returns all documents
db.<collection>.find({<field>: <value>})                           --- Returns documents matching a filter
db.<collection>.findOne()                                          --- Returns one document
db.<collection>.insertOne({<field>: <value>})                      --- Adds a document
db.<collection>.updateOne({<filter>}, {$set: {<field>: <value>}})  --- Updates one document
db.<collection>.deleteOne({<filter>})                              --- Deletes one document
db.<collection>.countDocuments()                                   --- Counts documents

# ---Indexes & Cleanup---
db.<collection>.createIndex({<field>: 1})  --- Creates an index
db.<collection>.getIndexes()               --- Lists a collection's indexes
db.<collection>.drop()                     --- Deletes a collection
db.dropDatabase()                          --- Deletes the current database

# ---Backup & Restore---
mongodump --uri="<uri>" --out=<dir>                           --- Exports a database to a folder
mongorestore --uri="<uri>" <dir>                              --- Restores from a dump folder
mongoexport --db=<db> --collection=<name> --out=<file.json>   --- Exports a collection to JSON
mongoimport --db=<db> --collection=<name> --file=<file.json>  --- Imports a JSON file into a collection
```

### Redis

**Flag:** `-redis`  |  **Also matches:** redis-cli, cache  |  **Platforms:** Linux, macOS

> **Note:** Redis has no official native Windows build. On Windows, run it through WSL or Docker

#### Everything Redis

```text
# ---Connect & Basics---
redis-cli                      --- Connects to a local server
redis-cli -h <host> -p <port>  --- Connects to a remote server
redis-cli -a <password>        --- Connects with a password
PING                           --- Checks that the server responds
SELECT <db>                    --- Switches to a numbered database

# ---Keys & Expiry---
SET <key> <value>       --- Stores a value
GET <key>               --- Reads a value
DEL <key>               --- Deletes a key
EXISTS <key>            --- Checks whether a key exists
EXPIRE <key> <seconds>  --- Sets a key to expire
TTL <key>               --- Shows the seconds before a key expires
SCAN 0 MATCH <pattern>  --- Finds keys matching a pattern
TYPE <key>              --- Shows a key's data type

# ---Data Types---
INCR <key>                  --- Adds 1 to a number
HSET <key> <field> <value>  --- Sets a field in a hash
HGETALL <key>               --- Reads all fields of a hash
LPUSH <key> <value>         --- Adds a value to the start of a list
LRANGE <key> 0 -1           --- Reads a whole list
SADD <key> <member>         --- Adds a member to a set
SMEMBERS <key>              --- Lists the members of a set

# ---Admin & Debug---
INFO      --- Shows server information
DBSIZE    --- Counts the keys in the database
MONITOR   --- Streams every command the server receives
FLUSHDB   --- Deletes every key in the database
FLUSHALL  --- Deletes every key in every database
```
---

## Linux & System Tools

### Permissions & Users

**Flag:** `-perms`  |  **Also matches:** chmod, chown, sudo, users, groups  |  **Platforms:** Linux, macOS

> **Note:** The Users & Groups commands are Linux-specific. macOS manages users differently

#### Everything Permissions

```text
# ---Set File Permissions---
chmod +x <file>            --- Makes a file executable
chmod 755 <file>           --- Sets rwxr-xr-x (owner full, others read and run)
chmod 644 <file>           --- Sets rw-r--r-- (owner edits, others read)
chmod 600 <file>           --- Sets rw------- (owner only)
chmod -R 755 <folder>      --- Applies a mode to a folder and its contents
chmod u+rwx,go-rwx <file>  --- Gives the owner everything and others nothing

# ---Change Ownership---
chown <user> <file>                 --- Changes a file's owner
chown <user>:<group> <file>         --- Changes the owner and group
chown -R <user>:<group> <folder>    --- Changes ownership of a folder and its contents
sudo chown -R $USER:$USER <folder>  --- Takes ownership of a folder
chgrp <group> <file>                --- Changes a file's group

# ---Inspect Permissions---
ls -l        --- Lists files with their permissions
stat <file>  --- Shows detailed file info
id           --- Shows your user and group IDs
groups       --- Lists your groups
umask        --- Shows the default permission mask

# ---Users & Groups---
[linux] sudo useradd -m <user>           --- Creates a user with a home folder
[linux] sudo passwd <user>               --- Sets a user's password
[linux] sudo usermod -aG <group> <user>  --- Adds a user to a group
[linux] sudo userdel -r <user>           --- Deletes a user and their home folder
su - <user>                              --- Switches to another user
sudo -i                                  --- Opens a root shell
whoami                                   --- Prints your username
```

### Find & Search Text

**Flag:** `-find`  |  **Also matches:** grep, rg, ripgrep, search-files  |  **Platforms:** Linux, macOS

#### Everything Search

```text
# ---Find Files---
find . -name "<pattern>"                        --- Finds files by name
find . -iname "<pattern>"                       --- Finds files by name, ignoring case
find . -type f -name "<pattern>"                --- Finds only files
find . -type d -name "<name>"                   --- Finds only folders
find . -size +100M                              --- Finds files larger than 100 MB
find . -mtime -7                                --- Finds files changed in the last 7 days
find . -name "<pattern>" -exec <command> {} \;  --- Runs a command on each match
find . -name "*.log" -delete                    --- Deletes matching files
find . -empty                                   --- Finds empty files and folders

# ---Search Text (grep)---
grep "<text>" <file>                  --- Finds lines containing text
grep -rn "<text>" .                   --- Searches all files with line numbers
grep -i "<text>" <file>               --- Searches ignoring case
grep -v "<text>" <file>               --- Shows lines that do not match
grep -c "<text>" <file>               --- Counts matching lines
grep -E "<regex>" <file>              --- Searches with an extended regex
grep -l "<text>" *                    --- Lists files that contain the text
grep -rn "<text>" --include="*.js" .  --- Searches only certain file types
grep -A 3 -B 3 "<text>" <file>        --- Shows 3 lines around each match

# ---Search Faster (ripgrep)---
rg "<text>"        --- Searches the current folder recursively
rg -i "<text>"     --- Searches ignoring case
rg -t js "<text>"  --- Searches only one file type
rg -l "<text>"     --- Lists files that contain the text
rg --files         --- Lists the files ripgrep would search
```

### sed, awk & Text Tools

**Flag:** `-sed`  |  **Also matches:** awk, sort, uniq, wc, head, tail, cut, pipe  |  **Platforms:** Linux, macOS

> **Note:** On macOS, sed -i needs an empty suffix: sed -i '' 's/<old>/<new>/g' <file>

#### Everything Text

```text
# ---Replace Text (sed)---
sed 's/<old>/<new>/' <file>          --- Replaces the first match on each line
sed 's/<old>/<new>/g' <file>         --- Replaces every match
sed -i 's/<old>/<new>/g' <file>      --- Edits the file in place
sed -i.bak 's/<old>/<new>/g' <file>  --- Edits in place and keeps a backup
sed -n '10,20p' <file>               --- Prints lines 10 to 20
sed '/<pattern>/d' <file>            --- Deletes lines matching a pattern

# ---Columns & Calculations (awk)---
awk '{print $1}' <file>                 --- Prints the first column
awk -F',' '{print $2}' <file.csv>       --- Prints the second column of a CSV
awk '/<pattern>/ {print}' <file>        --- Prints lines matching a pattern
awk '{sum+=$1} END {print sum}' <file>  --- Adds up the first column
awk 'NR==<n>' <file>                    --- Prints one line by number

# ---Sort, Count & Slice---
sort <file>           --- Sorts lines alphabetically
sort -n <file>        --- Sorts lines numerically
sort -u <file>        --- Sorts and removes duplicates
uniq -c               --- Counts repeated lines (input must be sorted)
wc -l <file>          --- Counts lines
head -n 10 <file>     --- Shows the first 10 lines
tail -n 20 <file>     --- Shows the last 20 lines
tail -f <file>        --- Follows a file as it grows
cut -d',' -f1 <file>  --- Prints the first comma-separated field
diff <file1> <file2>  --- Shows the differences between two files

# ---Pipes & Redirection---
<command> | <command>             --- Sends one command's output to another
<command> > <file>                --- Writes output to a file, replacing it
<command> >> <file>               --- Appends output to a file
<command> 2> <file>               --- Writes error messages to a file
<command> > <file> 2>&1           --- Writes output and errors to a file
sort <file> | uniq -c | sort -nr  --- Counts and ranks repeated lines
```

### Archives (tar & zip)

**Flag:** `-archive`  |  **Also matches:** tar, zip, unzip, gzip, 7z, compress  |  **Platforms:** Windows, Linux, macOS

> **Note:** tar is built in to Windows 10 and later. zip, unzip, gzip, and gunzip are not, so use tar or PowerShell there. 7z needs 7-Zip installed (p7zip on Linux)

#### Everything Archives

```text
# ---tar Archives---
tar -czf <archive.tar.gz> <folder>   --- Creates a gzip-compressed archive
tar -xzf <archive.tar.gz>            --- Extracts a .tar.gz archive
tar -xzf <archive.tar.gz> -C <dest>  --- Extracts into a chosen folder
tar -tzf <archive.tar.gz>            --- Lists an archive's contents
tar -cJf <archive.tar.xz> <folder>   --- Creates an xz-compressed archive
tar -xf <archive>                    --- Extracts any tar archive, detecting compression

# ---zip & gzip---
[unix] zip -r <archive.zip> <folder>   --- Creates a zip archive
[unix] unzip <archive.zip>             --- Extracts a zip archive
[unix] unzip <archive.zip> -d <dest>   --- Extracts into a chosen folder
[unix] unzip -l <archive.zip>          --- Lists a zip's contents
[unix] gzip <file>                     --- Compresses a single file
[unix] gunzip <file.gz>                --- Decompresses a .gz file
[win] tar -a -c -f <archive.zip> <folder>  --- Creates a zip archive
[win] tar -xf <archive.zip>                --- Extracts a zip archive

# ---7-Zip---
7z a <archive.7z> <folder>  --- Creates a 7z archive
7z x <archive.7z>           --- Extracts a 7z archive
7z l <archive>              --- Lists an archive's contents

# ---PowerShell Zip (Windows)---
[win] Compress-Archive -Path <folder> -DestinationPath <archive.zip>  --- Creates a zip archive
[win] Expand-Archive -Path <archive.zip> -DestinationPath <dest>      --- Extracts a zip archive
```
### systemd & journalctl

**Flag:** `-systemd`  |  **Also matches:** systemctl, journalctl, service, daemon  |  **Platforms:** Linux

#### Everything systemd

```text
# ---Control Services---
systemctl status <service>             --- Shows a service's status
sudo systemctl start <service>         --- Starts a service
sudo systemctl stop <service>          --- Stops a service
sudo systemctl restart <service>       --- Restarts a service
sudo systemctl reload <service>        --- Reloads a service's config
sudo systemctl enable --now <service>  --- Starts a service now and at every boot
sudo systemctl disable <service>       --- Stops a service starting at boot

# ---List & Debug Services---
systemctl list-units --type=service       --- Lists running services
systemctl list-unit-files --type=service  --- Lists all installed services
systemctl --failed                        --- Lists services that failed
systemctl cat <service>                   --- Shows a service's unit file
systemctl is-active <service>             --- Prints whether a service is running
systemctl list-timers                     --- Lists scheduled timers

# ---Create a Service---
sudo nano /etc/systemd/system/<name>.service  --- Creates or edits a service file
sudo systemctl daemon-reload                  --- Reloads unit files after changes
sudo systemctl enable --now <name>            --- Starts the new service and enables it at boot
systemctl --user status <service>             --- Shows a user-level service's status
systemctl --user enable --now <service>       --- Starts and enables a user-level service

# ---Read Logs (journalctl)---
journalctl -u <service>                       --- Shows a service's logs
journalctl -u <service> -f                    --- Follows a service's logs live
journalctl -u <service> --since "1 hour ago"  --- Shows logs from the last hour
journalctl -b                                 --- Shows logs since the last boot
journalctl -p err                             --- Shows only errors
journalctl -xe                                --- Jumps to the latest entries with explanations
journalctl -k                                 --- Shows kernel messages

# ---Log Space & Power---
journalctl --disk-usage           --- Shows how much space logs use
sudo journalctl --vacuum-time=7d  --- Deletes logs older than 7 days
sudo reboot                       --- Restarts the machine
sudo shutdown now                 --- Shuts down immediately
sudo shutdown -r +10              --- Restarts in 10 minutes
shutdown -c                       --- Cancels a scheduled shutdown
```

### Cron

**Flag:** `-cron`  |  **Also matches:** crontab, scheduled-tasks, schedule  |  **Platforms:** Linux, macOS

#### Everything Cron

```text
# ---Manage Cron Jobs---
crontab -e                 --- Edits your cron jobs
crontab -l                 --- Lists your cron jobs
crontab -r                 --- Removes all your cron jobs
sudo crontab -u <user> -l  --- Lists another user's cron jobs

# ---Schedule Formats---
* * * * * <command>     --- Runs every minute
0 * * * * <command>     --- Runs at the start of every hour
0 2 * * * <command>     --- Runs every day at 2:00 AM
0 9 * * 1-5 <command>   --- Runs at 9:00 AM on weekdays
*/15 * * * * <command>  --- Runs every 15 minutes
0 0 1 * * <command>     --- Runs on the first day of every month
@reboot <command>       --- Runs once at startup

# ---Debug Cron---
grep CRON /var/log/syslog     --- Shows cron activity (Debian and Ubuntu)
journalctl -u cron            --- Shows cron logs on systemd systems
<command> >> <log-file> 2>&1  --- Saves a job's output and errors to a log
```

### Disk & Storage

**Flag:** `-disk`  |  **Also matches:** df, du, lsblk, mount, ncdu, diskutil, free-space  |  **Platforms:** Linux, macOS

> **Note:** Drive and partition tools differ. Linux uses lsblk, fdisk, and mkfs. macOS uses diskutil. ncdu is a separate install

#### Everything Disk

```text
# ---Check Free Space---
df -h                             --- Shows free space on each drive
du -sh <folder>                   --- Shows the total size of a folder
[linux] du -h --max-depth=1       --- Shows the size of each subfolder
[mac] du -h -d 1                  --- Shows the size of each subfolder
du -ah . | sort -rh | head -n 20  --- Lists the 20 largest files and folders
ncdu <folder>                     --- Browses disk usage interactively

# ---Drives & Partitions---
[linux] lsblk          --- Lists drives and partitions
[linux] lsblk -f       --- Lists drives with file system types
[linux] sudo fdisk -l  --- Lists partition tables
[linux] blkid          --- Shows drive UUIDs and file system types
[mac] diskutil list    --- Lists drives and partitions
[mac] diskutil info <disk>  --- Shows a drive's details and file system

# ---Mount & Unmount---
[linux] sudo mount <device> <folder>  --- Mounts a drive on a folder
[linux] sudo umount <folder>          --- Unmounts a drive
[linux] findmnt                       --- Shows mounts as a tree
[mac] diskutil mount <disk>           --- Mounts a drive
[mac] diskutil unmount <disk>         --- Unmounts a drive
mount                                 --- Lists everything that is mounted

# ---Format & Check---
[linux] sudo mkfs.ext4 <device>                    --- Formats a drive as ext4 (erases it)
[linux] sudo fsck <device>                         --- Checks and repairs a file system
[mac] diskutil eraseDisk APFS <name> <disk>        --- Formats a whole drive as APFS (erases it)
[mac] diskutil verifyVolume <volume>               --- Checks a volume's file system
lsof +D <folder>                                   --- Lists processes using files in a folder
```
### Shell Productivity (Bash)

**Flag:** `-bash`  |  **Also matches:** terminal, zsh, history, alias  |  **Platforms:** Linux, macOS

> **Note:** macOS uses zsh by default, so its config file is ~/.zshrc. watch is not preinstalled on macOS. tldr is a separate install on both systems

#### Everything Shell

```text
# ---History & Shortcuts---
history  --- Lists your command history
!!       --- Repeats the last command
sudo !!  --- Repeats the last command with sudo
Ctrl+r   --- Searches your command history
Ctrl+c   --- Stops the running command
Ctrl+z   --- Pauses the running command
Ctrl+l   --- Clears the screen
Ctrl+a   --- Jumps to the start of the line
Ctrl+e   --- Jumps to the end of the line

# ---Aliases, Variables & Config---
alias <name>='<command>'  --- Creates a shortcut for a command
export <NAME>=<value>     --- Sets an environment variable
echo $<NAME>              --- Prints a variable's value
source ~/.bashrc          --- Reloads your Bash config
source ~/.zshrc           --- Reloads your Zsh config
which <command>           --- Shows where a command lives
type <command>            --- Shows how a command is interpreted

# ---Jobs & Background Tasks---
<command> &        --- Runs a command in the background
jobs               --- Lists background jobs
fg                 --- Brings a job to the foreground
bg                 --- Resumes a paused job in the background
nohup <command> &  --- Keeps a command running after you log out
disown             --- Detaches a job from the shell

# ---Help & Timing---
man <command>               --- Opens a command's manual
<command> --help            --- Shows a command's usage summary
tldr <command>              --- Shows short practical examples (needs install)
[mac] brew install watch    --- Installs watch, which macOS lacks
watch -n 2 <command>        --- Reruns a command every 2 seconds
time <command>              --- Measures how long a command takes
```
### tmux

**Flag:** `-tmux`  |  **Also matches:** terminal-multiplexer, sessions, panes  |  **Platforms:** Linux, macOS

#### Everything tmux

```text
# ---Sessions---
tmux new -s <name>                  --- Starts a named session
tmux ls                             --- Lists sessions
tmux attach -t <name>               --- Reattaches to a session
tmux kill-session -t <name>         --- Ends a session
tmux rename-session -t <old> <new>  --- Renames a session
Ctrl+b d                            --- Detaches from the session

# ---Windows & Panes (Keys)---
Ctrl+b c        --- Creates a new window
Ctrl+b n        --- Goes to the next window
Ctrl+b p        --- Goes to the previous window
Ctrl+b ,        --- Renames the current window
Ctrl+b %        --- Splits the pane side by side
Ctrl+b "        --- Splits the pane top and bottom
Ctrl+b <arrow>  --- Moves between panes
Ctrl+b z        --- Zooms the current pane
Ctrl+b x        --- Closes the current pane
Ctrl+b [        --- Enters scroll mode (q to exit)

# ---Config & Help---
tmux source-file ~/.tmux.conf  --- Reloads your tmux config
tmux list-keys                 --- Lists all key bindings
Ctrl+b ?                       --- Shows key bindings inside tmux
tmux kill-server               --- Ends every session and tmux itself
```

### Vim

**Flag:** `-vim`  |  **Also matches:** vi, neovim, nvim  |  **Platforms:** Windows, Linux, macOS

#### Everything Vim

```text
# ---Save & Quit---
:w   --- Saves the file
:q   --- Quits
:wq  --- Saves and quits
:q!  --- Quits without saving
:x   --- Saves and quits if there are changes
i    --- Enters insert mode
Esc  --- Returns to normal mode

# ---Move Around---
h j k l  --- Moves left, down, up, right
w        --- Jumps to the next word
b        --- Jumps to the previous word
0        --- Jumps to the start of the line
$        --- Jumps to the end of the line
gg       --- Jumps to the top of the file
G        --- Jumps to the bottom of the file
:<n>     --- Jumps to line number n
%        --- Jumps to the matching bracket
Ctrl+f   --- Scrolls down a page
Ctrl+b   --- Scrolls up a page

# ---Edit Text---
dd      --- Deletes the current line
yy      --- Copies the current line
p       --- Pastes after the cursor
u       --- Undoes the last change
Ctrl+r  --- Redoes the last undone change
ciw     --- Changes the word under the cursor
o       --- Opens a new line below and enters insert mode
A       --- Appends at the end of the line
.       --- Repeats the last change

# ---Search & Replace---
/<text>             --- Searches forward
n                   --- Jumps to the next match
N                   --- Jumps to the previous match
*                   --- Searches for the word under the cursor
:%s/<old>/<new>/g   --- Replaces every match in the file
:%s/<old>/<new>/gc  --- Replaces matches, asking each time
:noh                --- Clears search highlighting

# ---Select, Windows & Files---
v          --- Starts character selection
V          --- Starts line selection
Ctrl+v     --- Starts block selection
:split     --- Splits the window horizontally
:vsplit    --- Splits the window vertically
Ctrl+w w   --- Switches between windows
:e <file>  --- Opens another file
:ls        --- Lists open buffers
```

### nano

**Flag:** `-nano`  |  **Also matches:** pico  |  **Platforms:** Linux, macOS

> **Note:** nano is not built in to Windows. Use it through Git Bash or WSL. The nano that ships with macOS is old, so Alt shortcuts may not work until you run brew install nano

#### Everything nano

```text
# ---Open & Save---
nano <file>  --- Opens a file in nano
Ctrl+o       --- Saves the file
Ctrl+x       --- Exits nano
Ctrl+g       --- Opens the help screen

# ---Edit Text---
Ctrl+k  --- Cuts the current line
Ctrl+u  --- Pastes the cut text
Alt+a   --- Starts or ends a selection
Alt+6   --- Copies the selection or current line
Alt+u   --- Undoes the last change
Alt+e   --- Redoes the last undone change

# ---Search & Navigate---
Ctrl+w  --- Searches for text
Ctrl+\  --- Searches and replaces text
Ctrl+_  --- Jumps to a line number
Ctrl+a  --- Jumps to the start of the line
Ctrl+e  --- Jumps to the end of the line
```
### curl & wget

**Flag:** `-curl`  |  **Also matches:** wget, http, api-request, download  |  **Platforms:** Windows, Linux, macOS

> **Note:** In Windows PowerShell, curl is an alias for Invoke-WebRequest. Use curl.exe instead

#### Everything curl

```text
# ---Send GET Requests---
curl <url>     --- Fetches a URL and prints the response
curl -i <url>  --- Includes the response headers
curl -I <url>  --- Shows only the response headers
curl -L <url>  --- Follows redirects
curl -s <url>  --- Hides the progress output
curl -v <url>  --- Shows the full request and response details

# ---Send POST & JSON Requests---
[unix] curl -X POST <url> -H "Content-Type: application/json" -d '{"key":"value"}'     --- Sends a JSON body
[win] curl -X POST <url> -H "Content-Type: application/json" -d "{\"key\":\"value\"}"  --- Sends a JSON body
curl -X POST <url> -d "<key>=<value>"                                                  --- Sends form data
curl -F "file=@<path>" <url>                                                           --- Uploads a file
curl -X PUT <url> -d "<data>"                                                          --- Sends a PUT request
curl -X DELETE <url>                                                                   --- Sends a DELETE request

# ---Headers & Authentication---
curl -H "Authorization: Bearer <token>" <url>  --- Sends a bearer token
curl -u <user>:<password> <url>                --- Sends basic authentication
curl -H "<header>: <value>" <url>              --- Adds a custom header
curl -c <cookie-file> <url>                    --- Saves cookies to a file
curl -b <cookie-file> <url>                    --- Sends cookies from a file

# ---Download Files (curl)---
curl -o <file> <url>             --- Saves the response to a named file
curl -O <url>                    --- Saves the file under its remote name
curl -C - -O <url>               --- Resumes an interrupted download
curl --max-time <seconds> <url>  --- Gives up after a time limit
curl -k <url>                    --- Ignores TLS certificate errors

# ---Download Files (wget)---
wget <url>            --- Downloads a file
wget -O <file> <url>  --- Saves the download under a chosen name
wget -c <url>         --- Resumes an interrupted download
wget -i <urls.txt>    --- Downloads every URL listed in a file
wget -r -np <url>     --- Downloads a folder recursively
```

---

## Package Managers

### apt (Debian & Ubuntu)

**Flag:** `-apt`  |  **Also matches:** dpkg, ubuntu, debian  |  **Platforms:** Linux

#### Everything apt

```text
# ---Install & Remove---
sudo apt update         --- Refreshes the package list
sudo apt install <pkg>  --- Installs a package
sudo apt remove <pkg>   --- Removes a package
sudo apt purge <pkg>    --- Removes a package and its config files
sudo apt autoremove     --- Removes packages nothing needs anymore

# ---Update the System---
sudo apt upgrade       --- Installs available updates
sudo apt full-upgrade  --- Upgrades and handles changing dependencies
apt list --upgradable  --- Lists packages with updates
sudo apt clean         --- Clears the downloaded package cache

# ---Search & Inspect---
apt search <term>                   --- Searches for packages
apt show <pkg>                      --- Shows a package's details
apt list --installed                --- Lists installed packages
dpkg -L <pkg>                       --- Lists the files a package installed
dpkg -S <file>                      --- Finds which package owns a file
sudo dpkg -i <file.deb>             --- Installs a .deb file
sudo add-apt-repository ppa:<name>  --- Adds a PPA repository
```

### pacman & paru (Arch)

**Flag:** `-pacman`  |  **Also matches:** paru, aur, arch, cachyos, manjaro  |  **Platforms:** Linux

#### Everything pacman

```text
# ---Install & Remove---
sudo pacman -S <pkg>    --- Installs a package
sudo pacman -Rns <pkg>  --- Removes a package, its config, and unused dependencies
paru -S <pkg>           --- Installs a package from the AUR
paru -Rns <pkg>         --- Removes an AUR package

# ---Update & Clean---
sudo pacman -Syu                   --- Updates the whole system
paru -Syu                          --- Updates the system and AUR packages
sudo pacman -Sc                    --- Clears old cached packages
pacman -Qdtq | sudo pacman -Rns -  --- Removes orphans (errors if none exist)

# ---Search & Inspect---
pacman -Ss <term>  --- Searches the repositories
pacman -Qs <term>  --- Searches installed packages
pacman -Qi <pkg>   --- Shows an installed package's details
pacman -Ql <pkg>   --- Lists the files a package installed
pacman -Qo <file>  --- Finds which package owns a file
pacman -Qe         --- Lists packages you installed explicitly
pacman -Qdtq       --- Lists orphaned packages
```

### dnf (Fedora & RHEL)

**Flag:** `-dnf`  |  **Also matches:** fedora, rpm, yum  |  **Platforms:** Linux

> **Note:** Fedora 41 and later use dnf5. Every line here also works on dnf4 (RHEL 9 and older)

#### Everything dnf

```text
# ---Install & Remove---
sudo dnf install <pkg>       --- Installs a package
sudo dnf remove <pkg>        --- Removes a package
sudo dnf autoremove          --- Removes packages nothing needs anymore
sudo dnf install <file.rpm>  --- Installs a local .rpm file

# ---Update the System---
sudo dnf upgrade    --- Updates all packages
dnf check-upgrade   --- Lists available updates
sudo dnf clean all  --- Clears cached package data
dnf history         --- Lists past transactions

# ---Search & Inspect---
dnf search <term>     --- Searches for packages
dnf info <pkg>        --- Shows a package's details
dnf list --installed  --- Lists installed packages
dnf provides <file>   --- Finds which package provides a file
rpm -ql <pkg>         --- Lists the files a package installed
```

### Homebrew

**Flag:** `-brew`  |  **Also matches:** homebrew, cask  |  **Platforms:** Linux, macOS

#### Everything Homebrew

```text
# ---Install & Remove---
brew install <pkg>               --- Installs a package
[mac] brew install --cask <app>  --- Installs a macOS app
brew uninstall <pkg>             --- Removes a package
brew cleanup                     --- Removes old versions and cache files

# ---Update---
brew update         --- Updates Homebrew's package list
brew upgrade        --- Upgrades all installed packages
brew upgrade <pkg>  --- Upgrades one package
brew outdated       --- Lists packages with updates
brew doctor         --- Checks your setup for problems

# ---Search & Inspect---
brew search <term>      --- Searches for packages
brew info <pkg>         --- Shows a package's details
brew list               --- Lists installed packages
brew deps --tree <pkg>  --- Shows a package's dependencies

# ---Background Services---
brew services list               --- Lists managed services
brew services start <service>    --- Starts a service and keeps it at login
brew services stop <service>     --- Stops a service
brew services restart <service>  --- Restarts a service
```

### winget

**Flag:** `-winget`  |  **Also matches:** windows-package-manager  |  **Platforms:** Windows

#### Everything winget

```text
# ---Install & Remove---
winget install <pkg>                      --- Installs a package
winget install --id <id>                  --- Installs by exact package ID
winget install <pkg> --version <version>  --- Installs a specific version
winget uninstall <pkg>                    --- Removes a package

# ---Update---
winget upgrade        --- Lists packages with updates
winget upgrade <pkg>  --- Upgrades one package
winget upgrade --all  --- Upgrades everything
winget source update  --- Refreshes the package sources

# ---Search & Inspect---
winget search <term>          --- Searches for packages
winget show <pkg>             --- Shows a package's details
winget list                   --- Lists installed programs
winget export -o <file.json>  --- Saves your installed packages to a file
winget import -i <file.json>  --- Installs packages from a saved file
```

### Flatpak & Snap

**Flag:** `-flatpak`  |  **Also matches:** snap, flathub  |  **Platforms:** Linux

#### Everything Flatpak

```text
# ---Flatpak---
flatpak install flathub <app-id>  --- Installs an app from Flathub
flatpak run <app-id>              --- Runs an app
flatpak list                      --- Lists installed apps
flatpak update                    --- Updates all apps
flatpak uninstall <app-id>        --- Removes an app
flatpak uninstall --unused        --- Removes unused runtimes

# ---Snap---
sudo snap install <pkg>  --- Installs a snap
snap list                --- Lists installed snaps
sudo snap refresh        --- Updates all snaps
sudo snap remove <pkg>   --- Removes a snap
snap find <term>         --- Searches for snaps
```

---

## Version Control

### Git (additions to v1)

**Flag:** `-git`  |  **Also matches:** rebase, submodule, worktree, reflog, git-advanced  |  **Platforms:** Windows, Linux, macOS

> **Note:** These sections are added to the existing Git tool. The v1 Git sections stay unchanged

#### Rewrite History

```text
# ---Rebase & Edit History---
git rebase <branch>                --- Replays your commits on top of another branch
git rebase -i HEAD~<n>             --- Edits the last n commits interactively
git rebase --continue              --- Continues after fixing conflicts
git rebase --abort                 --- Cancels the rebase
git rebase --skip                  --- Skips the current commit
git commit --fixup <commit>        --- Creates a fix for an earlier commit
git rebase -i --autosquash <base>  --- Applies fixup commits automatically
git pull --rebase                  --- Pulls and rebases your commits

# ---Amend & Fix Commits---
git commit --amend                       --- Edits the last commit
git commit --amend --no-edit             --- Adds staged changes to the last commit
git commit --amend -m "<message>"        --- Rewrites the last commit message
git commit --allow-empty -m "<message>"  --- Creates a commit with no changes
git reset --soft HEAD~<n>                --- Undoes commits and keeps the changes staged
git push --force-with-lease              --- Safely force-pushes after rewriting history
```

#### Repositories & Branches

```text
# ---Submodules---
git submodule add <url> <path>           --- Adds a repository inside this one
git clone --recurse-submodules <url>     --- Clones a repo with its submodules
git submodule update --init --recursive  --- Fetches submodules after a plain clone
git submodule update --remote            --- Updates submodules to their latest commit
git submodule status                     --- Shows each submodule's commit
git submodule deinit -f <path>           --- Unregisters a submodule

# ---Worktrees---
git worktree add <path> <branch>         --- Checks out a branch in a new folder
git worktree add -b <new-branch> <path>  --- Creates a branch in a new folder
git worktree list                        --- Lists all worktrees
git worktree remove <path>               --- Removes a worktree
git worktree prune                       --- Cleans up deleted worktrees

# ---Branches, Tracking & Cleanup---
git fetch --all --prune               --- Updates every remote and drops deleted branches
git branch -vv                        --- Shows each branch's upstream
git branch -r                         --- Lists remote branches
git checkout --track origin/<branch>  --- Creates a local branch tracking a remote one
git push -u origin <branch>           --- Pushes a new branch and sets its upstream
git branch --merged                   --- Lists branches already merged
git remote prune origin               --- Deletes stale remote-tracking branches
```

#### Recovery & Search

```text
# ---Recover Lost Work---
git reflog                            --- Lists where HEAD has been
git reset --hard <commit>             --- Moves the branch back to a commit
git branch <name> <commit>            --- Creates a branch at a lost commit
git cherry-pick <commit>              --- Applies a single commit to this branch
git restore --source=<commit> <file>  --- Restores a file from an old commit
git fsck --lost-found                 --- Finds unreachable commits

# ---Search History---
git log -S"<text>"                 --- Finds commits that added or removed text
git log --grep="<text>"            --- Finds commits by message
git log --author="<name>"          --- Shows commits by one author
git log -- <file>                  --- Shows the history of one file
git log --follow -- <file>         --- Shows a file's history across renames
git blame -L <start>,<end> <file>  --- Shows who changed specific lines
git shortlog -sn                   --- Counts commits per author
```

#### Configuration

```text
# ---Git Config Tweaks---
git config --global init.defaultBranch main    --- Sets main as the default branch name
git config --global pull.rebase true           --- Makes git pull rebase by default
git config --global push.autoSetupRemote true  --- Sets the upstream automatically on first push
git config --global core.editor "<editor>"     --- Sets your commit message editor
git config --global credential.helper store    --- Saves credentials to a file
git config --global alias.<name> "<command>"   --- Creates a git shortcut
git config --global core.autocrlf input        --- Normalizes line endings on commit
git config --global --edit                     --- Opens your global config file

# ---Ignore Files & Untrack---
git rm --cached <file>      --- Stops tracking a file but keeps it on disk
git rm -r --cached .        --- Untracks everything so .gitignore can apply
git check-ignore -v <file>  --- Shows which rule ignores a file
git status --ignored        --- Shows ignored files in the status
git clean -fdx              --- Deletes untracked and ignored files for good
```

---

## 5. Open items for IMPLEMENTATIONV2.md

1. **Help output shape:** `ezcmd --help` lists categories and tools. `ezcmd --help -<tool>` lists that tool's groups and section titles. `ezcmd --help -<tool> -<section>` shows commands.
2. **Section lookup without a tool:** `ezcmd --help -<section>` searches every tool. Several matches go through the `[1/2/3]` prompt, and each choice should show its tool (for example `Flutter: Run the App`).
3. **Aliases:** write at least 3 per section, and add the tool's `Also matches` names as tool aliases.
4. **Tool flags:** the `-<slug>` names above must not collide with each other or with OS names (`-windows`, `-linux`, `-macos`).
5. **Platform tags:** extend the v1 `data/*.toml` format with `windows`, `linux`, `macos`, and `unix` keys (already supported) and a tool-level `platforms` field.
6. **Verification:** see section 6 below. Everything not listed as verified should be checked against current documentation before release.

---

## 6. Verification status

**Checked against current documentation or reference material on October 18, 2026:**

- Flutter: `flutter run` interactive keys, `flutter emulators --create`, `flutter build web --wasm`, `flutter run -d chrome --wasm`, `flutter create . --platforms web`
- Expo and React Native: EAS CLI commands (`eas init`, `eas update`, `eas build --local`, channels), React Native CLI (`init`, `doctor`, `start`, `run-android`, `run-ios`)
- Next.js: `create-next-app` options (TypeScript is now the default), `next dev`, `next start`, `next info`, `next telemetry`, `next build --webpack`
- JavaScript tooling: npm (`explain`, `audit signatures`, `--omit=dev`, `cache verify`), Yarn Berry (`up`, `dlx`, `workspaces foreach -A`, `set version stable`, `--immutable`), Bun (`--watch run`, `build --compile`, `pm cache rm`, `create`), Vercel CLI (`rollback`, `promote`, `env pull`, `domains add`)
- Python: uv (`python pin`, `pip compile`, `tool upgrade --all`, `self update`, `lock --upgrade`), Poetry 2 (`env activate`, `shell` moved to a plugin)
- Backend services: Supabase CLI (`db diff -f`, `migration list`, `gen types typescript`, `secrets`, `functions`), Firebase CLI (`emulators:start --import --export-on-exit`, `functions:secrets:set`, `hosting:channel:deploy`)
- Editors: nano shortcuts

**Corrections made after checking:**

- `create-next-app`: `--ts` is the default, so it is now labeled that way, with `--no-ts`, `--yes`, and `--use-bun` added
- Flutter hot reload keys: removed `P` and `v`, which I could not confirm, and added the confirmed `s`, `b`, `i`, `w`
- `flutter emulators --create`: note corrected (creates a Pixel-based Android emulator) and the optional `--name` added
- Next.js telemetry: replaced the unconfirmed `status` subcommand with `--enable` and `--disable`
- Poetry: added `poetry env activate`, `poetry env use`, and the plugin that restores `poetry shell`
- Yarn: `create` is labeled Classic, with the Berry equivalent `yarn dlx create-...` added
- nano: replaced `Ctrl+6` with the confirmed `Alt+a`, and added `Alt+6` for copy
- Small additions: `vercel promote`, `vercel pull`, `eas update --auto`, `eas update:rollback`, `uv lock --upgrade`, Supabase `--linked` and `secrets unset`, Firebase `--export-on-exit`

**Checked in the third pass (October 18, 2026), against current documentation:**

- Flutter: `flutter run` key list (r, R, h, d, c, q, s, b, w, i, o), `dart devtools`, and the `flutter create --template` values (app, module, package, plugin, plugin_ffi, skeleton)
- Android: `adb shell monkey` app launch syntax
- Expo: `eas update:rollback`, `eas update:list`, `eas channel:list`, `eas build:list`, `eas build:configure` (all present in eas-cli)
- Netlify CLI: `env:list`, `env:set`, `env:import`, `functions:create`, `functions:invoke` (all present in the current reference)
- Vercel CLI: `vercel logs <deployment-url>`, `rollback`, `promote`, `inspect`
- Supabase CLI: `gen types` syntax in the current CLI reference
- dnf: command names in the dnf5 manual
- TypeScript: TypeScript 7.0 is released and `tsc` is now the native compiler

**Corrections made in the third pass:**

- `adb shell monkey -p <package> 1` did not reliably launch the app. It now includes `-c android.intent.category.LAUNCHER`
- Supabase: `gen types typescript --local` is the old form. The current CLI uses `gen types --lang typescript --local` (same for `--linked` and `--project-id`)
- dnf: `dnf list installed` is wrong on dnf5 (Fedora 41+). It is now `dnf list --installed`, which works on dnf4 and dnf5. `dnf check-update` is now `dnf check-upgrade`, the dnf5 name (dnf4 keeps it as an alias). Added a tool note about dnf5
- Flutter: `--template=package` creates a reusable Flutter package, not a Dart package, so the note now says that
- TypeScript: added a tool note that TypeScript 7 ships the native compiler as `tsc` and the 6.0 compatibility package provides `tsc6`

**Still unconfirmed:** whether `tsc --init` behaves the same on the TypeScript 7.0 native compiler (an earlier preview did not support it). Check this before release.

**Not individually re-checked** (stable, widely documented commands): Docker, Docker Compose, kubectl, GitHub CLI, SSH, rsync, Terraform, AWS, gcloud, Azure, the other ADB and Android SDK tools, Gradle, Maven, CocoaPods, Xcode tools, Python and pip basics, Rust and Cargo, Go, Java, the database clients, Linux command-line tools, the other package managers, and the Git additions.

**Fourth pass (October 18, 2026), platform and accuracy review from reference knowledge, not a fresh documentation check:**

- Disk & Storage: tagged Linux-only commands `[linux]`, added macOS equivalents (`diskutil`, `du -h -d 1`)
- Nginx & Certbot: tagged systemd and `/var/log` lines `[linux]`, added Homebrew equivalents for macOS, switched reload to `nginx -s reload`
- Archives: tagged `zip`, `unzip`, `gzip`, `gunzip` as `[unix]`, added Windows `tar` zip commands, updated the tool note
- Xcode: `xcodebuild test` now includes `-destination`, added `-showdestinations`
- Shell Productivity: added `source ~/.zshrc`, noted `watch` and `tldr` installs
- Redis: platforms changed to Linux and macOS, with a Windows note (WSL or Docker)
- nano: platforms changed to Linux and macOS, with a Windows note
- Vite: added the `@vitest/coverage-v8` install line before `--coverage`
- pacman: orphan removal note now says it errors when no orphans exist
- winget: removed `scoop` and `choco` from Also matches
- Aliases: removed `shell` from Bash (the v1 `shell` tool owns that slug) and `editor` from Vim and nano (two tools cannot share an alias)

**Checked against live documentation after the fourth pass (October 18, 2026):**

- Supabase: `supabase gen types --lang typescript` with `--local`, `--linked`, and `--project-id` matches the current CLI reference, so the existing lines stay as written
- TypeScript: the `@typescript/typescript6` package providing `tsc6` is confirmed, and the tool note now names it
- TypeScript: `npx tsc <file.ts>` fails in a folder with a tsconfig.json under TypeScript 7, so the line now uses `--ignoreConfig`

**Still unconfirmed:** `tsc --init` on the TypeScript 7 native compiler (none of the sources I checked mention it, so check it by running it on 7.0), and whether the macOS-bundled nano (2.0.6 as of macOS High Sierra) supports the `Alt` shortcuts on current macOS. Flutter was reviewed and left unchanged, though `flutter run -d chrome --wasm` needs a recent Flutter and `flutter screenshot` is worth a quick check.
