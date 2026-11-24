# Integration branch `integ/next`

Open pull requests of Marvisak/VIctory merged together on `main` (`ffe310e`), each head unchanged, and tested together. Not an upstream branch: a place to test the combination and to measure it on the title's shaders.

## How it was built

MamaLeVrai's [BUNDLE] stack (#1319 → #1402 → #1492 → #1504 → #1511, at their heads of 2026-11-24: #1402 `119b7630`, #1492 `5bbc0985` without #98, #1504 `1d418919`, #1511 `ca601f02`), then #704, #180, #309, #160, the wave64 stack (#223 → #233 → #320 → #439), the GPU bundles #1418, #1426, #1439, then #493 and #431, then #334, #346, #1024, #211 and #492 on top of this file's first version. Every conflict was resolved by hand; each merge commit says what each side did and how they were combined, including the changes git did not flag (signatures changed by one side, tests that would have been nested and never run, behaviour one side undid). Integration-only commits are named as such.

This is the third build of the line (`integ/next-v3`). The previous one sat on the bundles' earlier heads, whose #1492 carried #98 (it loses 29 programs on the title's shaders, zed6000 on #98 and #1247); every merge was replayed on the current #1511 with the same resolutions (git rerere trained on the previous merges, then each result checked against the previous merge commit file by file). Outside the bundles' own changes (#98 gone; #1402's `119b7630` skips two of #306's tests below validator 1.8) the tree is the previous build's.

Merge order (first parent):

1. `a793ad86 victory-shader: lower all-zero and all-one constant lane masks`
1. `5bd27c89 Merge commit 'refs/pull/1074/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `5853aff7 Merge commit 'refs/pull/1084/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `c62973bf Merge commit 'refs/pull/953/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `8cd85559 Merge commit 'refs/pull/1056/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `a7e83d15 Merge commit 'refs/pull/1015/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `57379e0a Merge commit 'refs/pull/563/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `d542c830 Merge commit 'refs/pull/1055/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `a562c1a6 Merge commit 'refs/pull/1004/head' of https://github.com/Marvisak/VIctory into bundle/lane-masks-scalar-reads`
1. `643aef64 Bundle: lane-mask, scalar descriptor read and cube-sample shader PRs, verified together`
1. `fc064297 Merge commit 'refs/pull/1067/head' of https://github.com/Marvisak/VIctory into bundle/shader-high-yield`
1. `a01fc762 Merge commit 'refs/pull/305/head' of https://github.com/Marvisak/VIctory into bundle/shader-high-yield`
1. `9c38bd77 Merge commit 'refs/pull/306/head' of https://github.com/Marvisak/VIctory into bundle/shader-high-yield`
1. `75516e77 Merge commit 'refs/pull/176/head' of https://github.com/Marvisak/VIctory into bundle/shader-high-yield`
1. `23104092 Bundle: high-yield shader PRs on top of #1319, verified together`
1. `119b7630 Bundle high-yield: skip two of #306's tests when dxil.dll is older than validator 1.8`
1. `7b1455c4 Merge commit '344704e02850eb227fe61d4fd54e6657c17ea68d' into trial/shader-lowering-c`
1. `d053700e Merge commit '5d6889e8be76f630445a4057f81221b723898583' into trial/shader-lowering-c`
1. `77e3d29e Merge commit '9483b1a962613cf91d30ac2a2d491657ed08f5c9' into trial/shader-lowering-c`
1. `34826e71 Merge commit '645391bcce957054e39f2d3df52177062769283e' into trial/shader-lowering-c`
1. `3cc282bc Merge commit 'f47ebd83f314e2680130bb3854975120bf1eaa69' into trial/shader-lowering-c`
1. `f0f7e136 Merge commit '5ecb6abe769d2507a64324637c34d1597ab27e39' into trial/shader-lowering-c`
1. `d146ad8e Merge commit '4b8a1ca6dd0d30a7c39d6d633d97e850dae2bccf' into trial/shader-lowering-c`
1. `bea7e635 Merge commit '1052a90a407242812ea02caffe806b92c5f462ae' into trial/shader-lowering-c`
1. `ada65a7e Merge commit '8d67fa5b00eb6714c5ac71e9d01172cc6cdbcf08' into trial/shader-lowering-c`
1. `a95185b5 Merge commit '3d71ad1a6547836ac1dbd19bfab2d759c965d307' into trial/shader-lowering-c`
1. `a8a631bd Merge commit '7b57e8e8da92ba1e690f5bffc19ed64edec58ce0' into trial/shader-lowering-c`
1. `0443f28e Merge commit '41b2472e134c8f402cacc011001dec1a8ca294e4' into trial/shader-lowering-c`
1. `a624dc74 Merge commit '393320dfa8cde9ef1744e9cc8931d64ba0e954a8' into trial/shader-lowering-c`
1. `6a1fcd41 Merge commit 'd25783b8d87dea346cccd4f8aff90d6dd75f9258' into trial/shader-lowering-c`
1. `c75ee332 Merge commit 'bf873ffd3b30ea77b18736e63a032cf33d2dc7e6' into trial/shader-lowering-c`
1. `5fb25849 Merge commit 'd160854be046f2d70a71ade4ba9836f0c6edcd15' into trial/shader-lowering-c`
1. `483258bf Merge commit '7dffb2a6c245c1ee68efd602be5cd0934e9ee0f9' into trial/shader-lowering-c`
1. `df1fb77f Merge commit 'ba532339235f404ae6d60b6cb39152373422c1fc' into trial/shader-lowering-c`
1. `1470fadc Merge commit '5be1fe4b658d8e3ae623122f410366ea3369f8bc' into trial/shader-lowering-c`
1. `8263ed57 Merge commit '6a9cc8fef867562687f90b3fd70a74322fe615c9' into trial/shader-lowering-c`
1. `5bbc0985 Bundle: shader instruction lowering PRs on top of #1402, verified together`
1. `1d418919 Merge PR #453 (computed barycentrics) into the shader stack, with the resolution of its conflicts with #176 and #1293`
1. `0c7da166 Merge commit 'ea633e723d3c502c777b95780056700c0af71b75' into trial/shader-dxil4`
1. `89bd94d2 Merge commit '9459c12d2e3a3d48ac2e073e7d3d9a888bfa913d' into trial/shader-dxil4`
1. `bb85b047 Merge commit 'ad330155a5a031e5632a95c622e4d39a31e32d88' into trial/shader-dxil4`
1. `538fb4b8 Merge commit '428725aaed223953709bcc7d526f3070a424587d' into trial/shader-dxil4`
1. `fcc5fb18 Merge commit 'bd59ba346da72d106ef47c2c837779ef8d8e5abd' into trial/shader-dxil4`
1. `ca601f02 Bundle: DXIL emission and container robustness PRs on top of #1504, verified together`
1. `8f129126 Merge #704 (unknown-stride buffers through a raw view) onto #1511`
1. `5703405e Merge #180 (twin descriptor loads, quad gradients) onto #704 + #1511`
1. `c50c4e7c Merge #309 (SGPRs reloaded from VGPR lanes) onto #180 + #704 + #1511`
1. `0bb7335e Tests: #309's negative control after the bundle tracks spilled masks`
1. `b821b311 Merge #160 (texel offsets, comparison and gather forms of image samples) onto #309 + #180 + #704 + #1511`
1. `364f08b1 Merge remote-tracking branch 'refs/remotes/pr/1418' into integ/next-v3`
1. `22f78dd3 Merge bundle #1426 (GPU host copies, descriptors, sampler heaps, footprints) onto #1418 + shader stack`
1. `73397be1 Merge bundle #1439 (pipeline library, shader cache, draw state, frame wait) onto #1426 + #1418 + shader stack`
1. `09d5bbd0 Merge #493 onto the GPU bundles (#1439 + #1426 + #1418) + shader stack`
1. `3703c1d5 Merge #431 onto the GPU bundles (#1439 + #1426 + #1418) + shader stack`
1. `d398dde3 Merge the wave64 stack line (fc30a7d7: #223, #233, #320, #439) with the GPU line`
1. `26d5b80c Docs: INTEGRATION.md for the integration branch (PRs, merge order, validation)`
1. `a6f86332 Merge #334 (host copies of title memory transitioned between shader writes and reads) onto integ/next`
1. `feb919c7 Merge #346 (indirect rectangle-list draws expanded on the GPU) onto #334 + integ/next`
1. `b07c6758 Merge #1024 (copy-back writes only the GPU's bytes, marks only unchanged pages current) onto #346 + #334 + integ/next`
1. `31a600fb Merge #211 (PresentX planes composited; Series S black UI/menus) onto #1024 + #346 + #334 + integ/next`
1. `967d3aec Merge #492 (R9G9B9E5_SHAREDEXP targets and UAVs made as R16G16B16A16_FLOAT) onto #211 + #1024 + #346 + #334 + integ/next`

## Validation (Windows 11, RTX 4060 Laptop, 32-lane waves, driver 32.0.16.1047; WARP)

Runner `scripts/validate.py` (run `rebuild-v3`), on the last merge (`967d3aec`); this file's commit changes nothing else.

| check | result |
|---|---|
| `cargo test --workspace --locked --no-fail-fast` (RTX) | 571 passed, 0 failed, 8 ignored |
| same with `VICTORY_WARP=1` | 570 passed, 1 failed (besides the flaky pool test below): `emit::valu_tests::v_cndmask_on_an_untracked_mask_tests_this_lanes_bit` (from #1084: it pins a wave size, and with `VICTORY_WARP=1` its "hardware" pass is WARP too, which refuses fixed `WaveSize` pipelines; test harness only) |
| `cargo test --workspace --all-features` (RTX) | 580 passed (adds the `diagnostics` tests) |
| `cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask policy` | clean |

Against the previous build (572 / 571 + 1 / 581): the one test that leaves is #98's `emit::wave::tests::permlane16_reads_the_lanes_own_row`; every other test has the same outcome. The two #306 tests that #1402's `119b7630` skips below validator 1.8 run here (this `dxil.dll` is 1.8 or later). `host::batch::upload_pool_reuses_buffers_and_handles_varying_sizes` (#982, via #1426) can fail when the workspace runs in parallel: it expects the upload buffer it returned to the shared host's pool to come back to it, and another test can take it first. It failed in the runner's WARP pass on `967d3aec` (so that pass read 569 passed, 2 failed) and in one of three `cargo test -p victory-gpu --lib` reruns on WARP; it passes alone (5 of 5) and with `--test-threads=1`. `victory-gpu` is byte-identical to the previous build, so this is not from the rebuild.

After each merge the number of tests that run was checked against the number the PR adds.

## Not measured

- The combination on the title's shaders (`victory translate --validate --list`): offline gains in #1247 were measured per PR on `main`. Known overlap: #309's spilled-mask idiom is already covered by the bundle stack, so its gain here is lower than +125.
- Rendering in the world, and anything on AMD or Intel hardware.

## Not included

- #98 (v_permlane16 in whole-wave sections): no longer in bundle #1492; it loses 29 programs on the title's shaders according to zed6000 (#98, #1247).
- #282 (indirect rectangle-list draws expanded on the CPU, #255): the same draws as #346, which is taken instead (only one can land; the reasons are in the #346 merge commit).
- #693 (shadow buffers in DEFAULT VRAM with staging readback): incompatible with #1024, which is taken instead; #693 has unfixed in-title regressions (about 2x slower frames on an RX 6700 XT, https://github.com/Marvisak/VIctory/pull/693#issuecomment-7136073380; GPU page faults in freed staging on RX 6950 XT/6700 XT, https://github.com/Marvisak/VIctory/pull/693#issuecomment-7135206500).
- #306's latest commit (`e3067aea`, after the `93421530b` the bundle carries): not in #1402, so #306 is not listed below although the bundle carries the rest of it.

## Pull requests whose head is contained here (109)

| PR | author | head | via |
|---|---|---|---|
| #24 | artor410 | `d5f41f54c` | #1439 |
| #55 | a55uka | `56f1d4138` | #1426 |
| #72 | FIocker | `521ceccd7` | #1439 |
| #119 | ifBars | `095e44a0b` | #1418 |
| #123 | 9hvn9yt2vk-netizen | `bc747281d` | #1426 |
| #124 | TG0000 | `a6deb46db` | #1418 |
| #143 | AndyLatimer | `525a713e8` | #1418 |
| #153 | TG0000 | `3e98bd7e0` | #1418 |
| #156 | Rareuncommon | `344704e02` | #1492 |
| #160 | r4tedx | `5ce85c5e7` | direct |
| #162 | Rareuncommon | `5d6889e8b` | #1492 |
| #168 | Dufftwt | `9483b1a96` | #1492 |
| #169 | Dufftwt | `645391bcc` | #1492 |
| #174 | AndyLatimer | `882664e13` | #1439 |
| #176 | Yaekai | `70278092a` | #1402 |
| #180 | permadiego | `aa692580e` | direct |
| #193 | n-romanovich | `f99b792e0` | direct |
| #202 | ifBars | `dcf666486` | #1426 |
| #208 | wbkepler | `eb723596b` | #1426 |
| #211 | Yaekai | `2fb7a8b44` | direct |
| #223 | r4tedx | `714b81805` | direct |
| #233 | r4tedx | `ededb1bab` | direct |
| #270 | Maraco91 | `92f2c60c8` | #1418 |
| #278 | Quat3rnion | `d063cee54` | #1418 |
| #305 | tarcisioads | `78dae8362` | #1402 |
| #309 | AntoninMichelq | `64c7188a1` | direct |
| #320 | r4tedx | `41369c902` | direct |
| #322 | jsthope | `b58d198d9` | #1426 |
| #334 | realhenners | `c99e4290b` | direct |
| #343 | pedrofort | `9459c12d2` | #1511 |
| #346 | BastienRodz | `8ef6bfea1` | direct |
| #347 | AugustoZFJ | `197c7acf1` | #1439 |
| #352 | rhuantac | `f77bd28c6` | #1439 |
| #358 | Dufftwt | `369dc3246` | #1439 |
| #368 | Quat3rnion | `f47ebd83f` | #1492 |
| #379 | LouisRiverstone | `5ecb6abe7` | #1492 |
| #409 | ToxicmaggotRP | `340499c44` | #1426 |
| #427 | GamingRipperDoc | `16d395ccc` | #1439 |
| #431 | AntoninMichelq | `23ced703f` | direct |
| #435 | melon0918 | `657efd827` | #1426 |
| #439 | r4tedx | `28aca3b71` | direct |
| #453 | AntoninMichelq | `88b34a447` | #1504 |
| #456 | Loggerten | `4b8a1ca6d` | #1492 |
| #474 | nathanbzrr | `e34cb5396` | #1426 |
| #477 | nathanbzrr | `bd59ba346` | #1511 |
| #484 | ToxicmaggotRP | `3b7e1d5f9` | #1418 |
| #485 | fuzeofficlal | `d454508f7` | #1439 |
| #488 | enzoduarte1 | `fba33207c` | #1418 |
| #492 | Yaekai | `c7bb83fd3` | direct |
| #493 | hophaver | `c48b240ad` | direct |
| #515 | desplmfao | `d9cb90856` | #1418 |
| #550 | edsuuu | `8560817ad` | #1439 |
| #553 | broccolize | `b796c8b8b` | #1418 |
| #563 | SAitken91 | `10ac6ceac` | #1319 |
| #587 | FibbS6 | `1052a90a4` | #1492 |
| #626 | LAKSHYA-cmd-tech | `df9ab3c69` | #1439 |
| #631 | shayanCSknu | `e009e13aa` | #1418 |
| #642 | nathanbzrr | `6b987062d` | #1439 |
| #704 | r4tedx | `1f3c184a3` | direct |
| #933 | enzoduarte1 | `9914ea4be` | #1426 |
| #934 | muhammed-aksoy | `95cefdaca` | #1418 |
| #953 | MartinIndra02 | `c42cbb073` | #1319 |
| #982 | chemokita13 | `b4abe5db6` | #1426 |
| #990 | LouisRiverstone | `8d67fa5b0` | #1492 |
| #1000 | TG0000 | `f821cb9d7` | #1439 |
| #1004 | tarcisioads | `cbc8c33dc` | #1319 |
| #1015 | justrealyme | `b8cfe8ce7` | #1319 |
| #1021 | CaravelaLabs | `3d71ad1a6` | #1492 |
| #1024 | Dufftwt | `67125b3a8` | direct |
| #1032 | divyeshganta | `b38a3c9fa` | #1426 |
| #1042 | gabrielcruzg3 | `ad330155a` | #1511 |
| #1048 | LouisRiverstone | `7b57e8e8d` | #1492 |
| #1052 | LouisRiverstone | `41b2472e1` | #1492 |
| #1053 | LouisRiverstone | `393320dfa` | #1492 |
| #1055 | adamdotpy | `21d1b4682` | #1319 |
| #1056 | adamdotpy | `417f4e30d` | #1319 |
| #1067 | undeadhuman37 | `6d30fd71c` | #1402 |
| #1073 | undeadhuman37 | `986fa6db9` | #1319 |
| #1074 | undeadhuman37 | `ec337a025` | #1319 |
| #1084 | undeadhuman37 | `48a41fb85` | #1319 |
| #1095 | ESAT586 | `d25783b8d` | #1492 |
| #1129 | divyeshganta | `d09c31b4c` | #1439 |
| #1142 | ESAT586 | `bf873ffd3` | #1492 |
| #1145 | hamdog0203 | `ea633e723` | #1511 |
| #1146 | jamesn-xt | `47c00d5ea` | #1418 |
| #1152 | pablogonz12 | `e8fd20f85` | #1439 |
| #1158 | Vittorio17 | `abefa4920` | #1418 |
| #1190 | ESAT586 | `d160854be` | #1492 |
| #1191 | geiyiliyabumo | `1456fe76c` | #1418 |
| #1203 | geiyiliyabumo | `76291ef38` | #1439 |
| #1204 | geiyiliyabumo | `e3c9dd0d5` | #1418 |
| #1206 | geiyiliyabumo | `ef0c3e944` | #1426 |
| #1211 | ESAT586 | `7dffb2a6c` | #1492 |
| #1217 | neonbyte1 | `a2e532d15` | #1426 |
| #1220 | BastienRodz | `b4e464738` | #1418 |
| #1223 | ESAT586 | `ba5323392` | #1492 |
| #1259 | pablogonz12 | `5be1fe4b6` | #1492 |
| #1279 | Mumogg | `4dd4cfc8c` | #1439 |
| #1283 | DoubleD17 | `b232b8838` | #1426 |
| #1293 | mgunes1311-lab | `6a9cc8fef` | #1492 |
| #1303 | geiyiliyabumo | `428725aae` | #1511 |
| #1319 | MamaLeVrai | `643aef646` | #1402 |
| #1402 | MamaLeVrai | `119b7630d` | #1492 |
| #1418 | MamaLeVrai | `39d044fa6` | direct |
| #1426 | MamaLeVrai | `3fc262e22` | direct |
| #1439 | MamaLeVrai | `f9eb9798e` | direct |
| #1492 | MamaLeVrai | `5bbc09850` | #1504 |
| #1504 | MamaLeVrai | `1d418919f` | #1511 |
| #1511 | MamaLeVrai | `ca601f028` | direct |

_Built with Claude Code (Claude Opus 5.5): conflict resolution, validation runner and this file. No game data._
