<p align="center">
  <p align="center">
   <img width="150" height="150" src="https://hips.hearstapps.com/hmg-prod/images/playful-golden-british-shorthair-cat-royalty-free-image-1701453627.jpg?crop=0.699xw:1.00xh;0.141xw,0" alt="Logo">
  </p>
	<h1 align="center"><b>EpiKodi</b></h1>
	<p align="center">
		The Kodi alternative.
    <br />
    <br />
    <b>Downloads for </b>
		<a href="https://github.com/Nestyles/EpiKodi/releases">macOS & Windows</a>
    <br />
  </p>
</p>
<br/>

EpiKodi is the alternative to Kodi. It's a media station that allows you to have details, play and share content automatically.

# Monorepo App Architecture

We use a combination of Rust, React, TypeScript, Tauri, SQLite, ChakraUI throughout this monorepo.

### Apps:

- `desktop`: A [Tauri](https://tauri.app) (Rust) app, using [React](https://react.dev/) on the frontend.

### License:
Portions of this software are licensed as follows:

- All third party components are licensed under the original license provided by the owner of the applicable component
- All other content not mentioned above is available under the MIT license as defined in [LICENSE](https://github.com/Nestyles/EpiKodi/blob/master/LICENSE)
  
# Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information. This guide is a work in progress, and is updated regularly as the app matures.

## Documentation

- **Technical Docs**: Developer and architecture documentation is in `tech_docs` (mdBook). Open it locally with:

```powershell
cd tech_docs
mdbook serve
```

- **User Docs**: End-user guides and walkthroughs are in `user_docs` (mdBook). Preview locally with:

```powershell
cd user_docs
mdbook serve
```

Both sets are included in the repository to keep documentation close to the code and easy to update when APIs or UX change.
