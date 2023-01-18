# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..v0.1.0) - 2023-01-18
### Package updates
- [gill-db](crates/gill-db) bumped to [gill-db-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-db-v0.1.0)
- [gill-markdown](crates/gill-markdown) bumped to [gill-markdown-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-markdown-v0.1.0)
- [gill-git-server](crates/gill-git-server) bumped to [gill-git-server-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-git-server-v0.1.0)
- [syntect-plugin](crates/syntect-plugin) bumped to [syntect-plugin-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..syntect-plugin-v0.1.0)
- [gill-web-markdown](crates/gill-web-markdown) bumped to [gill-web-markdown-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-web-markdown-v0.1.0)
- [gill-app](crates/gill-app) bumped to [gill-app-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-app-v0.1.0)
- [gill-settings](crates/gill-settings) bumped to [gill-settings-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-settings-v0.1.0)
- [gill-syntax](crates/gill-syntax) bumped to [gill-syntax-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-syntax-v0.1.0)
- [gill-git](crates/gill-git) bumped to [gill-git-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-git-v0.1.0)
### Global changes
#### Bug Fixes
- fix gh workflow path - ([9b0ea8a](https://github.com/oknozor/gill/commit/9b0ea8a42244965b8920155ffbaec0685499c9ee)) - [@oknozor](https://github.com/oknozor)
- user apub domain and url - ([a7021d3](https://github.com/oknozor/gill/commit/a7021d358262f5b31217e4b43b62b7a5d721da25)) - [@oknozor](https://github.com/oknozor)
#### Build system
- prepare github action - ([7e299d3](https://github.com/oknozor/gill/commit/7e299d3ada10e094cee3dabecba7bee9fb4e61b0)) - [@oknozor](https://github.com/oknozor)
- add production docker image - ([8bcdd72](https://github.com/oknozor/gill/commit/8bcdd72fb65809ce5199639a1b03d180675e567b)) - [@oknozor](https://github.com/oknozor)
#### Continuous Integration
- format packages in cog.toml - ([94fbaa8](https://github.com/oknozor/gill/commit/94fbaa8a211051c57aef1759973fd4b26ece4dd9)) - [@oknozor](https://github.com/oknozor)
- remove global set-version - ([e8e9461](https://github.com/oknozor/gill/commit/e8e9461823e4ea6384e9d6a619f921abbce3a226)) - [@oknozor](https://github.com/oknozor)
- add cog.toml - ([9879971](https://github.com/oknozor/gill/commit/9879971c870cdf63833db6d9a55c2676ee7bcf41)) - [@oknozor](https://github.com/oknozor)
- update checkout to v3 - ([96bcd10](https://github.com/oknozor/gill/commit/96bcd10b9202849026db31c22dde45caa57a0e3c)) - [@oknozor](https://github.com/oknozor)
#### Documentation
- add README - ([0ba7587](https://github.com/oknozor/gill/commit/0ba7587daa93de4be840241b19812733229b25d1)) - [@oknozor](https://github.com/oknozor)
#### Features
- **(activitypub)** toward a working apub impl - ([564423e](https://github.com/oknozor/gill/commit/564423ef6427fdda461a9fb85b904bcdacbc45c5)) - [@oknozor](https://github.com/oknozor)
- **(front)** add navbar skeleton - ([357918a](https://github.com/oknozor/gill/commit/357918a4c13cc91c5ee322e5fa4926f24c9ea2a2)) - [@oknozor](https://github.com/oknozor)
- implement activity pub ticket comment - ([e7cad5a](https://github.com/oknozor/gill/commit/e7cad5a48d5f9ba66c20b5e0ebdefe0ca6bf88bd)) - [@oknozor](https://github.com/oknozor)
- implement activity pub ticket - ([3a005e5](https://github.com/oknozor/gill/commit/3a005e5fa703073585f8146e1592146d4a2bec9d)) - [@oknozor](https://github.com/oknozor)
- improve git traversal and add commit info to tree view - ([f39650b](https://github.com/oknozor/gill/commit/f39650bf128dafb0ef4ad6cbd004ddc08b035ad5)) - [@oknozor](https://github.com/oknozor)
- canonicalize image links in markdown - ([3bbfb8a](https://github.com/oknozor/gill/commit/3bbfb8aeb08d031a5a5563aa1dfb41877a792f37)) - [@oknozor](https://github.com/oknozor)
- base pull requests - ([058d054](https://github.com/oknozor/gill/commit/058d0546320f3b184ca5096e1cde6a8bd80fe9cc)) - [@oknozor](https://github.com/oknozor)
- syntect diff again - ([dff84e5](https://github.com/oknozor/gill/commit/dff84e5c269702d09934505ff4e90a092a068143)) - [@oknozor](https://github.com/oknozor)
- fine grained ssh permission - ([3dc4b26](https://github.com/oknozor/gill/commit/3dc4b26f7f777a921245d99e9d3d0a32aa3807ce)) - [@oknozor](https://github.com/oknozor)
- implement base repository activities - ([733afcc](https://github.com/oknozor/gill/commit/733afcc9039a822fde766909a903c5b1fea4a09a)) - [@oknozor](https://github.com/oknozor)
- add templates for user profile and settings - ([2789ba7](https://github.com/oknozor/gill/commit/2789ba7d1b729af7e50e352f1cc20becaea41c78)) - [@oknozor](https://github.com/oknozor)
- reorganize submodules - ([1d8b5e4](https://github.com/oknozor/gill/commit/1d8b5e408b41f5a31277989fda2d7aa6cb17b8db)) - [@oknozor](https://github.com/oknozor)
- in memory assets - ([c135100](https://github.com/oknozor/gill/commit/c135100e2f3d53ed48fcbafe621c789b58c83dcc)) - [@oknozor](https://github.com/oknozor)
- repository history view - ([4a9f494](https://github.com/oknozor/gill/commit/4a9f4945592a083d058a729d903322ab72afd83f)) - [@oknozor](https://github.com/oknozor)
- add building blocks for apu commits - ([95ab3ff](https://github.com/oknozor/gill/commit/95ab3ff78d0f0f44702e7fd87e8750dc74d2bef2)) - [@oknozor](https://github.com/oknozor)
- add commit history for branch - ([af5515c](https://github.com/oknozor/gill/commit/af5515c300c03c00fd0a05b5e667edd514ec97a7)) - [@oknozor](https://github.com/oknozor)
- draft landing instance page - ([4dc047e](https://github.com/oknozor/gill/commit/4dc047e3a6e2aaea22563f5d02e5f939eb3d0149)) - [@oknozor](https://github.com/oknozor)
- add repository activities - ([1a98511](https://github.com/oknozor/gill/commit/1a98511bc2837473be239eb8c777893f55cb40eb)) - [@oknozor](https://github.com/oknozor)
- implement follow activity - ([c8ad38e](https://github.com/oknozor/gill/commit/c8ad38e2eb5e607e33f36109d34c5778a9e47fed)) - [@oknozor](https://github.com/oknozor)
- first apub interaction - ([25c3dcf](https://github.com/oknozor/gill/commit/25c3dcfc265ace5599106632640565437092ed6c)) - [@oknozor](https://github.com/oknozor)
- replace hljs with syntect - ([de88171](https://github.com/oknozor/gill/commit/de88171f3fcbbf0015a6a9b0020d0019bb70abdd)) - [@oknozor](https://github.com/oknozor)
- implement oauth for view - ([8b5f9a8](https://github.com/oknozor/gill/commit/8b5f9a8a847916dad70c68d604c1095769cced87)) - [@oknozor](https://github.com/oknozor)
- add template block - ([b4f92d8](https://github.com/oknozor/gill/commit/b4f92d8471b98fdf022d30364444f4c348995969)) - [@oknozor](https://github.com/oknozor)
- file explorer - ([60bd638](https://github.com/oknozor/gill/commit/60bd63879b4df1bdcd910ef4262932e28b0acc82)) - [@oknozor](https://github.com/oknozor)
- add git traversal naive impl - ([49a9cc8](https://github.com/oknozor/gill/commit/49a9cc8692b01dcbfc89bcc83e3bc72c6fb37c92)) - [@oknozor](https://github.com/oknozor)
- add askama templates - ([3f3d6a1](https://github.com/oknozor/gill/commit/3f3d6a152ddc614d56999f755be00ae8daaa1e38)) - [@oknozor](https://github.com/oknozor)
- list repositories - ([99d939d](https://github.com/oknozor/gill/commit/99d939d0a8e411d93399f0044d9b2145cccb07f3)) - [@oknozor](https://github.com/oknozor)
- oauth authentication for rest api - ([4505fd1](https://github.com/oknozor/gill/commit/4505fd18b4169857e37076461d12ab1add8b31a4)) - [@oknozor](https://github.com/oknozor)
- setup sqlx tests - ([0d51c1e](https://github.com/oknozor/gill/commit/0d51c1e446f6bba807652964edb9223c6aba280e)) - [@oknozor](https://github.com/oknozor)
- add dummy database model - ([271c804](https://github.com/oknozor/gill/commit/271c8042cd5a68e0796afce898e9190381e4bb43)) - [@oknozor](https://github.com/oknozor)
- add open api + some refactoring - ([972b1f5](https://github.com/oknozor/gill/commit/972b1f541f0afe53f54ece5e4c6f9e45d40646e7)) - [@oknozor](https://github.com/oknozor)
- git server base ops - ([c108109](https://github.com/oknozor/gill/commit/c1081096cabbefbf67839f1d367cbab6a1feee32)) - [@oknozor](https://github.com/oknozor)
#### Miscellaneous Chores
- include Cargo.lock - ([c493df7](https://github.com/oknozor/gill/commit/c493df76fdfe16eb7b1a34e0dd3998f8740e3a2d)) - [@oknozor](https://github.com/oknozor)
- update actiity-pub-federation rust - ([dd48114](https://github.com/oknozor/gill/commit/dd48114f71dabf2e4b5dd780142febf0bc585432)) - [@oknozor](https://github.com/oknozor)
- add readme badges - ([46af14f](https://github.com/oknozor/gill/commit/46af14f22471a60145151076ff10e26ac098510a)) - [@oknozor](https://github.com/oknozor)
- minify css - ([d2342fa](https://github.com/oknozor/gill/commit/d2342faf8156c9582a259abf585736fba2476733)) - [@oknozor](https://github.com/oknozor)
- add github sponsor - ([c7e940c](https://github.com/oknozor/gill/commit/c7e940c7343d475c53d40740186c2563548cac3e)) - [@oknozor](https://github.com/oknozor)
- add MIT License - ([56ac3b0](https://github.com/oknozor/gill/commit/56ac3b038ef38aed29cdd135ca4d96ad32987b9f)) - [@oknozor](https://github.com/oknozor)
- switch back to file assets for dev xp - ([10da582](https://github.com/oknozor/gill/commit/10da58287078c0a4ec44f30b35587783af063b8e)) - [@oknozor](https://github.com/oknozor)
- remove readme - ([7b75cb0](https://github.com/oknozor/gill/commit/7b75cb0d03cf2e3d7fdd37ab23829ce78c7133a7)) - [@oknozor](https://github.com/oknozor)
- add http2 axum feature, remove header check for repo root - ([4f42e5d](https://github.com/oknozor/gill/commit/4f42e5d6fbcba5afef33aaacf8357407a1e2bdf0)) - [@oknozor](https://github.com/oknozor)
- linguist, ignore assets - ([fb91f08](https://github.com/oknozor/gill/commit/fb91f080b2810e7688beaeb1be4308e5945925d0)) - [@oknozor](https://github.com/oknozor)
- delete unused files - ([0c81856](https://github.com/oknozor/gill/commit/0c818560246425797c402f3f994c23414e785a1d)) - [@oknozor](https://github.com/oknozor)
- slowly adding templates - ([5f96d95](https://github.com/oknozor/gill/commit/5f96d9529a9aef13cc3b6034dd21683c25d797d6)) - [@oknozor](https://github.com/oknozor)
- update axum and remove aide for now - ([eaa0a6d](https://github.com/oknozor/gill/commit/eaa0a6d61ee27f964698842b1d6eae1a0bbafc08)) - [@oknozor](https://github.com/oknozor)
- clippy lints - ([30c8da2](https://github.com/oknozor/gill/commit/30c8da26f660229daf51436a5a304511d6f0ae1f)) - [@oknozor](https://github.com/oknozor)
- cargo virtual workspace - ([3648efc](https://github.com/oknozor/gill/commit/3648efc89b1905b42ba77b97addc332f9b03a31a)) - [@oknozor](https://github.com/oknozor)
- fmt all - ([5cf74d2](https://github.com/oknozor/gill/commit/5cf74d2329f5e4122594c7a9fdadafbee6a4d1b3)) - [@oknozor](https://github.com/oknozor)
- change docker env - ([8066b2d](https://github.com/oknozor/gill/commit/8066b2d322e3b3e48cb736836443e66bb1e62341)) - [@oknozor](https://github.com/oknozor)
- rework dev environment to prepare for oauth integration - ([01bc3da](https://github.com/oknozor/gill/commit/01bc3da8eb9167d2af7ffd2846d0b4193180bef9)) - [@oknozor](https://github.com/oknozor)
#### Refactoring
- remove unused deps - ([9e67cbf](https://github.com/oknozor/gill/commit/9e67cbfe394411f9573b9a29eadb9b930670b49b)) - [@oknozor](https://github.com/oknozor)
- single app for all http endpoint - ([e50141f](https://github.com/oknozor/gill/commit/e50141f5549ff6c050051a37fa3a00a5d055dd75)) - [@oknozor](https://github.com/oknozor)
- extract database logic to its own module - ([f6dc7f9](https://github.com/oknozor/gill/commit/f6dc7f9ac1d4c5c2b0130d796b79b8f7add13660)) - [@oknozor](https://github.com/oknozor)
- move module app to lib - ([b382618](https://github.com/oknozor/gill/commit/b382618081cc8a6fb4dd5e77a216d7b5f7dfd3e6)) - [@oknozor](https://github.com/oknozor)
#### Tests
- fix tests - ([ddde273](https://github.com/oknozor/gill/commit/ddde2731da692745f27631e36c3c4267a6a66b1b)) - [@oknozor](https://github.com/oknozor)
- fix sqlx test compilation - ([0af6d9b](https://github.com/oknozor/gill/commit/0af6d9b001d2a86a092058e4a95736172e1d57cb)) - [@oknozor](https://github.com/oknozor)

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).