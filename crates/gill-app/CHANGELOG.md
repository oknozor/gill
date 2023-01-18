# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [gill-app-v0.1.0](https://github.com/oknozor/gill/compare/f81dee255ce5d86aad8119a44b8232153b30daca..gill-app-v0.1.0) - 2023-01-18
#### Bug Fixes
- **(css)** fix user profile header style - ([0b75027](https://github.com/oknozor/gill/commit/0b7502788e2afa300ed3c650db5089934c361fd8)) - [@oknozor](https://github.com/oknozor)
- fix ssh clone uri - ([55aea04](https://github.com/oknozor/gill/commit/55aea04eab6e43429c643e753f4709cadb6599d6)) - [@oknozor](https://github.com/oknozor)
- fix pr markdown description escape & highlithed line nth - ([07f90d5](https://github.com/oknozor/gill/commit/07f90d5b1aa33ebd54ff234fdbcd9bf9715ac9cf)) - [@oknozor](https://github.com/oknozor)
- typos - ([74ed983](https://github.com/oknozor/gill/commit/74ed983517a6dfa1ca37b2e41478abb3827ba68c)) - [@oknozor](https://github.com/oknozor)
- fix pull request state display - ([9a4c755](https://github.com/oknozor/gill/commit/9a4c7551ec145fb133068a35163390895e22d14d)) - [@oknozor](https://github.com/oknozor)
- encode branch uri component - ([19866ad](https://github.com/oknozor/gill/commit/19866ad7be01e9eaac6a4383bdb3f7f791084b1d)) - [@oknozor](https://github.com/oknozor)
#### Build system
- add production docker image - ([8bcdd72](https://github.com/oknozor/gill/commit/8bcdd72fb65809ce5199639a1b03d180675e567b)) - [@oknozor](https://github.com/oknozor)
#### Features
- **(app)** pull request comments - ([8a5ba1f](https://github.com/oknozor/gill/commit/8a5ba1fdbbcda93fd1690d70ccfcbddbb67db5b2)) - [@oknozor](https://github.com/oknozor)
- **(git)** add diff for a single commit - ([1434e08](https://github.com/oknozor/gill/commit/1434e08e5716b1274e8030407777370a2d6394b8)) - [@oknozor](https://github.com/oknozor)
- **(git)** implement rebase and merge - ([d86c685](https://github.com/oknozor/gill/commit/d86c685f728b4ff693fdb9979392c59b42f222a6)) - [@oknozor](https://github.com/oknozor)
- **(view)** generate commit author href and improve history view - ([e59ed77](https://github.com/oknozor/gill/commit/e59ed77f137b8b446626f60e217c694a56ce10f5)) - [@oknozor](https://github.com/oknozor)
- implement activity pub ticket comment - ([e7cad5a](https://github.com/oknozor/gill/commit/e7cad5a48d5f9ba66c20b5e0ebdefe0ca6bf88bd)) - [@oknozor](https://github.com/oknozor)
- implement activity pub ticket - ([3a005e5](https://github.com/oknozor/gill/commit/3a005e5fa703073585f8146e1592146d4a2bec9d)) - [@oknozor](https://github.com/oknozor)
- implement issues - ([f55696d](https://github.com/oknozor/gill/commit/f55696d5122e9056e545bcf554e82ee9725c08e1)) - [@oknozor](https://github.com/oknozor)
- improve markdown rendering style - ([505546a](https://github.com/oknozor/gill/commit/505546a9ae60c92316b4c2407d029af00a3167f1)) - [@oknozor](https://github.com/oknozor)
- improve several ui elements - ([bb855ca](https://github.com/oknozor/gill/commit/bb855ca4c64666209d91e2c0d802a34261869edd)) - [@oknozor](https://github.com/oknozor)
- improve git traversal and add commit info to tree view - ([f39650b](https://github.com/oknozor/gill/commit/f39650bf128dafb0ef4ad6cbd004ddc08b035ad5)) - [@oknozor](https://github.com/oknozor)
- rebase and merge from UI - ([a69b42d](https://github.com/oknozor/gill/commit/a69b42da3fd8fb350041914f065218f3505bedec)) - [@oknozor](https://github.com/oknozor)
- add markdown preview and wasm module - ([910c9d3](https://github.com/oknozor/gill/commit/910c9d32cf9ae5d476611d38135a8e8be32deb36)) - [@oknozor](https://github.com/oknozor)
- canonicalize image links in markdown - ([3bbfb8a](https://github.com/oknozor/gill/commit/3bbfb8aeb08d031a5a5563aa1dfb41877a792f37)) - [@oknozor](https://github.com/oknozor)
- base pull requests - ([058d054](https://github.com/oknozor/gill/commit/058d0546320f3b184ca5096e1cde6a8bd80fe9cc)) - [@oknozor](https://github.com/oknozor)
- use tailwind typography instead off custom style for readmes - ([26af8de](https://github.com/oknozor/gill/commit/26af8de6f0b87244bcb968abeb1bb44c279d53a5)) - [@oknozor](https://github.com/oknozor)
- syntect diff again - ([dff84e5](https://github.com/oknozor/gill/commit/dff84e5c269702d09934505ff4e90a092a068143)) - [@oknozor](https://github.com/oknozor)
- add repository navbar for issues, prs and code - ([ab41955](https://github.com/oknozor/gill/commit/ab419552f2e00995349ad0a334e254ce2c6f763c)) - [@oknozor](https://github.com/oknozor)
- sort tree and blobs in tree view - ([742f6e1](https://github.com/oknozor/gill/commit/742f6e14ccccfbbe1fc7d158060ce080d9236f9a)) - [@oknozor](https://github.com/oknozor)
- truncate branch name when too long on tree view - ([3561b78](https://github.com/oknozor/gill/commit/3561b788ff5a0204260fb9cb07ff5ec9f02a7e28)) - [@oknozor](https://github.com/oknozor)
- add ssh key from UI - ([d7fa4c9](https://github.com/oknozor/gill/commit/d7fa4c9a751ca6cc46cd6ec5bd7292fb67c76c23)) - [@oknozor](https://github.com/oknozor)
- fine grained ssh permission - ([3dc4b26](https://github.com/oknozor/gill/commit/3dc4b26f7f777a921245d99e9d3d0a32aa3807ce)) - [@oknozor](https://github.com/oknozor)
- add settings view skeleton - ([9580561](https://github.com/oknozor/gill/commit/9580561876e837d0f077e7388b1341fa91700aab)) - [@oknozor](https://github.com/oknozor)
- add support for image preview in treeview - ([1b55fc2](https://github.com/oknozor/gill/commit/1b55fc2f828c7235ff861dbae4e9c1499cefe807)) - [@oknozor](https://github.com/oknozor)
- simple diff with diff2html - ([c3d7f71](https://github.com/oknozor/gill/commit/c3d7f71bb8ba02f8e7dcee2fbe41dd7c7e537817)) - [@oknozor](https://github.com/oknozor)
- html diff - ([c9c6fc7](https://github.com/oknozor/gill/commit/c9c6fc7946fb78f9e3b18c078cdb07229d295dba)) - [@oknozor](https://github.com/oknozor)
- git diff - ([ab864e0](https://github.com/oknozor/gill/commit/ab864e078066f33f5a83b2fc0f581d890e2d26ab)) - [@oknozor](https://github.com/oknozor)
- implement base repository activities - ([733afcc](https://github.com/oknozor/gill/commit/733afcc9039a822fde766909a903c5b1fea4a09a)) - [@oknozor](https://github.com/oknozor)
- add templates for user profile and settings - ([2789ba7](https://github.com/oknozor/gill/commit/2789ba7d1b729af7e50e352f1cc20becaea41c78)) - [@oknozor](https://github.com/oknozor)
- in memory assets - ([c135100](https://github.com/oknozor/gill/commit/c135100e2f3d53ed48fcbafe621c789b58c83dcc)) - [@oknozor](https://github.com/oknozor)
#### Miscellaneous Chores
- update actiity-pub-federation rust - ([dd48114](https://github.com/oknozor/gill/commit/dd48114f71dabf2e4b5dd780142febf0bc585432)) - [@oknozor](https://github.com/oknozor)
- minify css - ([d2342fa](https://github.com/oknozor/gill/commit/d2342faf8156c9582a259abf585736fba2476733)) - [@oknozor](https://github.com/oknozor)
- clippy lints - ([7e1c51d](https://github.com/oknozor/gill/commit/7e1c51d14d725e56ee46b9ce7ab26bcbeb04e7ad)) - [@oknozor](https://github.com/oknozor)
- fmt & clippy - ([bc7b579](https://github.com/oknozor/gill/commit/bc7b57935fc2ec725f58610a0f56285725102043)) - [@oknozor](https://github.com/oknozor)
- switch back to file assets for dev xp - ([10da582](https://github.com/oknozor/gill/commit/10da58287078c0a4ec44f30b35587783af063b8e)) - [@oknozor](https://github.com/oknozor)
- thanks clippy - ([350fb65](https://github.com/oknozor/gill/commit/350fb653764636f393251bbd49d81a84d2ceb1fd)) - [@oknozor](https://github.com/oknozor)
#### Refactoring
- remove unused deps - ([9e67cbf](https://github.com/oknozor/gill/commit/9e67cbfe394411f9573b9a29eadb9b930670b49b)) - [@oknozor](https://github.com/oknozor)
- remove unused methods - ([cdb5aa9](https://github.com/oknozor/gill/commit/cdb5aa9100e4eae66b8b760d36039a4c69cfc025)) - [@oknozor](https://github.com/oknozor)
- add AppResult type - ([0797cf9](https://github.com/oknozor/gill/commit/0797cf9f82de845b91be1e7783aad0c24b1f59fd)) - [@oknozor](https://github.com/oknozor)
- add authorize derive macro - ([94ae7e4](https://github.com/oknozor/gill/commit/94ae7e42a14eaaf929a6b34ec9411294c9d84df3)) - [@oknozor](https://github.com/oknozor)
- move rebase and merge to domain - ([3df5bb9](https://github.com/oknozor/gill/commit/3df5bb9a56cf08533f1c7cb25e6f663f03a0d151)) - [@oknozor](https://github.com/oknozor)
- domain driven design step 1 - ([987c8a8](https://github.com/oknozor/gill/commit/987c8a8053010bcfd99d56f6e73f373bedb7765e)) - [@oknozor](https://github.com/oknozor)
- reorganize activity pub modules - ([c6cd274](https://github.com/oknozor/gill/commit/c6cd2740e2490298ca2058a41e997554b201895f)) - [@oknozor](https://github.com/oknozor)
- move tree template to subdir - ([60bd18b](https://github.com/oknozor/gill/commit/60bd18b906556273e8b3f556c7b655a017a7fc3d)) - [@oknozor](https://github.com/oknozor)
- remove duplication - ([6ab853d](https://github.com/oknozor/gill/commit/6ab853db9dab5ed27566e8a735c1bbff4c682d05)) - [@oknozor](https://github.com/oknozor)
- extract syntax highlight to a dedicated crate - ([0875ad9](https://github.com/oknozor/gill/commit/0875ad900f819309209827df89b6639444fa9006)) - [@oknozor](https://github.com/oknozor)
#### Tests
- fix tests - ([ddde273](https://github.com/oknozor/gill/commit/ddde2731da692745f27631e36c3c4267a6a66b1b)) - [@oknozor](https://github.com/oknozor)

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).