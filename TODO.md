## MVP
- [ ] Finish Console UI
  - [ ] Add config
    - [ ] Add installation
    - [ ] Remove installation
    - [ ] Add profile-symlink preference
  - [ ] First time launch - empty cache
    - [ ] Prompt to add installations
    - [ ] Check default locations

## Dev
- [ ] Logging for dev build
- [ ] Error handling, maybe panicking all the time is not ideal
- [ ] Custom Panic hook
- [ ] Create Profile trait
  - [ ] Change Vec<Profile> in Installation struct to Vec<&impl Profile>, to allow for diff. browser types to store custom data
    - [ ] Add distinction between profiles that can be added by name in Firefox (-P) -- Blocks adding -osint

## Goals
- [ ] Add GUI
- [ ] Custom browser type
- [ ] Last used profile
- [ ] URL filters
  - [ ] Adding -osint for Firefox
- [ ] Private windows support
- [ ] Error handling
- [ ] Linux support