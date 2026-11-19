use replicant_proto::StoryJumpPoint;

const AFTER_PROLOGUE_STEPS: [JumpStep; 17] = [
    JumpStep::SetFlag { id: 71 },
    JumpStep::SetFlag { id: 153 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::ClearFlag { id: 149 },
    JumpStep::SetTutorial { id: 2000 },
    JumpStep::SetTutorial { id: 2012 },
    JumpStep::SetTutorial { id: 2026 },
    JumpStep::SetTutorial { id: 2074 },
    JumpStep::SetTutorial { id: 2386 },
    JumpStep::SetTutorial { id: 2387 },
    JumpStep::SetTutorial { id: 2388 },
    JumpStep::SetTutorial { id: 2389 },
    JumpStep::SetTutorial { id: 2719 },
    JumpStep::SetTutorial { id: 2732 },
    JumpStep::SetItem { id: 727, count: 0 },
    JumpStep::SetItem { id: 1000, count: 1 },
    JumpStep::EquipWeapon { id: 1000 },
];

const POPOLA_TALK_EXTRA_STEPS: [JumpStep; 3] = [
    JumpStep::SetFlag { id: 154 },
    JumpStep::SetFlag { id: 1299 },
    JumpStep::SetFlag { id: 1401 },
];

const BEFORE_NEXT_DAY_EXTRA_STEPS: [JumpStep; 21] = [
    JumpStep::SetFlag { id: 154 },
    JumpStep::SetFlag { id: 73 },
    JumpStep::SetFlag { id: 156 },
    JumpStep::SetFlag { id: 160 },
    JumpStep::SetFlag { id: 161 },
    JumpStep::SetFlag { id: 157 },
    JumpStep::SetFlag { id: 1360 },
    JumpStep::SetFlag { id: 158 },
    JumpStep::SetFlag { id: 112 },
    JumpStep::SetFlag { id: 159 },
    JumpStep::SetFlag { id: 162 },
    JumpStep::SetFlag { id: 1443 },
    JumpStep::SetFlag { id: 165 },
    JumpStep::SetFlag { id: 163 },
    JumpStep::SetFlag { id: 164 },
    JumpStep::SetFlag { id: 166 },
    JumpStep::SetFlag { id: 1300 },
    JumpStep::ClearFlag { id: 1299 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetItem { id: 723, count: 1 },
    JumpStep::SetTutorial { id: 2010 },
];

const AT_SHRINE_ENTRANCE_EXTRA_STEPS: [JumpStep; 32] = [
    JumpStep::SetFlag { id: 167 },
    JumpStep::SetFlag { id: 168 },
    JumpStep::SetFlag { id: 169 },
    JumpStep::SetSnowFlag { id: 1052 },
    JumpStep::SetSnowFlag { id: 1069 },
    JumpStep::SetFlag { id: 170 },
    JumpStep::SetFlag { id: 171 },
    JumpStep::SetFlag { id: 172 },
    JumpStep::SetFlag { id: 173 },
    JumpStep::SetFlag { id: 174 },
    JumpStep::SetFlag { id: 175 },
    JumpStep::SetFlag { id: 176 },
    JumpStep::SetFlag { id: 189 },
    JumpStep::SetFlag { id: 190 },
    JumpStep::SetFlag { id: 191 },
    JumpStep::SetFlag { id: 116 },
    JumpStep::SetFlag { id: 77 },
    JumpStep::SetFlag { id: 1303 },
    JumpStep::ClearFlag { id: 1300 },
    JumpStep::ClearFlag { id: 1443 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::ClearFlag { id: 1404 },
    JumpStep::ClearFlag { id: 1405 },
    JumpStep::ClearFlag { id: 1406 },
    JumpStep::ClearFlag { id: 1407 },
    JumpStep::ClearFlag { id: 1408 },
    JumpStep::ClearFlag { id: 1409 },
    JumpStep::ClearFlag { id: 1410 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
];

const AT_SHRINE_TOP_EXTRA_STEPS: [JumpStep; 7] = [
    JumpStep::SetFlag { id: 75 },
    JumpStep::SetFlag { id: 192 },
    JumpStep::SetFlag { id: 138 },
    JumpStep::SetFlag { id: 193 },
    JumpStep::SetFlag { id: 1432 },
    JumpStep::SetFlag { id: 205 },
    JumpStep::SetItem { id: 305, count: 0 },
];

const AT_SHRINE_BOSS_EXTRA_STEPS: [JumpStep; 3] = [
    JumpStep::SetFlag { id: 194 },
    JumpStep::SetSnowFlag { id: 1053 },
    JumpStep::SetFlag { id: 206 },
];

const AT_SHRINE_ARENA_EXTRA_STEPS: [JumpStep; 9] = [
    JumpStep::ClearFlag { id: 1303 },
    JumpStep::SetFlag { id: 1432 },
    JumpStep::SetFlag { id: 1433 },
    JumpStep::SetFlag { id: 1434 },
    JumpStep::SetFlag { id: 127 },
    JumpStep::SetHaveWhiteBook { on: true },
    JumpStep::LearnMagic { id: 3 },
    JumpStep::SetMagicSlot { slot: 1, magic: 3 },
    JumpStep::SetTutorial { id: 2002 },
];

const AERIE_ENTRANCE_EXTRA_STEPS: [JumpStep; 35] = [
    JumpStep::SetFlag { id: 195 },
    JumpStep::SetFlag { id: 196 },
    JumpStep::SetFlag { id: 198 },
    JumpStep::SetFlag { id: 204 },
    JumpStep::SetFlag { id: 130 },
    JumpStep::LearnMagic { id: 6 },
    JumpStep::SetMagicSlot { slot: 0, magic: 6 },
    JumpStep::SetTutorial { id: 2005 },
    JumpStep::SetTutorial { id: 2020 },
    JumpStep::SetTutorial { id: 2079 },
    JumpStep::SetFlag { id: 200 },
    JumpStep::SetFlag { id: 201 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 202 },
    JumpStep::SetGameCounter { id: 100, value: 3 },
    JumpStep::SetGameCounter { id: 101, value: 3 },
    JumpStep::SetFlag { id: 217 },
    JumpStep::SetFlag { id: 219 },
    JumpStep::SetFlag { id: 1304 },
    JumpStep::SetFlag { id: 1361 },
    JumpStep::SetFlag { id: 218 },
    JumpStep::SetSnowFlag { id: 1085 },
    JumpStep::ClearFlag { id: 1304 },
    JumpStep::SetFlag { id: 220 },
    JumpStep::ClearFlag { id: 1305 },
    JumpStep::SetFlag { id: 221 },
    JumpStep::SetFlag { id: 222 },
    JumpStep::SetFlag { id: 224 },
    JumpStep::SetTutorial { id: 2014 },
    JumpStep::SetFlag { id: 225 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 239 },
];

const BOSS_FIGHT_3_LEAD_IN_EXTRA_STEPS: [JumpStep; 26] = [
    JumpStep::SetFlag { id: 70 },
    JumpStep::ClearFlag { id: 1433 },
    JumpStep::ClearFlag { id: 1434 },
    JumpStep::SetFlag { id: 79 },
    JumpStep::SetFlag { id: 1442 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::SetFlag { id: 240 },
    JumpStep::SetFlag { id: 241 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1409 },
    JumpStep::ClearFlag { id: 1409 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::SetFlag { id: 255 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1407 },
    JumpStep::ClearFlag { id: 1407 },
    JumpStep::SetFlag { id: 1408 },
    JumpStep::SetFlag { id: 242 },
    JumpStep::SetFlag { id: 1451 },
    JumpStep::ClearFlag { id: 1408 },
    JumpStep::SetFlag { id: 1409 },
    JumpStep::ClearFlag { id: 1409 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::SetFlag { id: 243 },
];

const BOSS_FIGHT_3_TO_JUNK_HEAP_FIRST_UPGRADE_EXTRA_STEPS: [JumpStep; 109] = [
    JumpStep::SetFlag { id: 244 },
    JumpStep::SetFlag { id: 245 },
    JumpStep::SetFlag { id: 246 },
    JumpStep::SetFlag { id: 247 },
    JumpStep::SetFlag { id: 248 },
    JumpStep::SetFlag { id: 129 },
    JumpStep::LearnMagic { id: 5 },
    JumpStep::SetTutorial { id: 2004 },
    JumpStep::ClearFlag { id: 1442 },
    JumpStep::SetFlag { id: 137 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 137 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetGameCounter { id: 100, value: 3 },
    JumpStep::SetGameCounter { id: 101, value: 3 },
    JumpStep::SetFlag { id: 249 },
    JumpStep::SetFlag { id: 252 },
    JumpStep::SetFlag { id: 253 },
    JumpStep::SetFlag { id: 1362 },
    JumpStep::SetFlag { id: 1357 },
    JumpStep::ClearFlag { id: 252 },
    JumpStep::SetFlag { id: 323 },
    JumpStep::SetFlag { id: 1355 },
    JumpStep::SetFlag { id: 1356 },
    JumpStep::SetFlag { id: 324 },
    JumpStep::SetFlag { id: 325 },
    JumpStep::ClearFlag { id: 1432 },
    JumpStep::ClearFlag { id: 1451 },
    JumpStep::SetFlag { id: 114 },
    JumpStep::SetFlag { id: 327 },
    JumpStep::SetFlag { id: 256 },
    JumpStep::SetTutorial { id: 2055 },
    JumpStep::SetItem { id: 1009, count: 1 },
    JumpStep::SetFlag { id: 328 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
    JumpStep::SetFlag { id: 81 },
    JumpStep::SetTutorial { id: 2021 },
    JumpStep::SetFlag { id: 337 },
    JumpStep::SetFlag { id: 1432 },
    JumpStep::ClearFlag { id: 1432 },
    JumpStep::SetFlag { id: 338 },
    JumpStep::SetFlag { id: 1316 },
    JumpStep::SetItem { id: 515, count: 1 },
    JumpStep::SetTutorial { id: 2022 },
    JumpStep::SetItem { id: 531, count: 1 },
    JumpStep::SetFlag { id: 339 },
    JumpStep::SetFlag { id: 1317 },
    JumpStep::SetFlag { id: 340 },
    JumpStep::ClearFlag { id: 1317 },
    JumpStep::SetFlag { id: 341 },
    JumpStep::SetFlag { id: 346 },
    JumpStep::SetGameCounter { id: 100, value: 2 },
    JumpStep::SetGameCounter { id: 101, value: 0 },
    JumpStep::ClearFlag { id: 1355 },
    JumpStep::ClearFlag { id: 1356 },
    JumpStep::ClearFlag { id: 1357 },
    JumpStep::SetFlag { id: 1363 },
    JumpStep::SetFlag { id: 358 },
    JumpStep::SetFlag { id: 360 },
    JumpStep::SetFlag { id: 361 },
    JumpStep::SetFlag { id: 268 },
    JumpStep::SetFlag { id: 269 },
    JumpStep::SetFlag { id: 365 },
    JumpStep::SetFlag { id: 271 },
    JumpStep::SetFlag { id: 272 },
    JumpStep::SetFlag { id: 277 },
    JumpStep::SetFlag { id: 275 },
    JumpStep::SetFlag { id: 276 },
    JumpStep::SetFlag { id: 280 },
    JumpStep::SetFlag { id: 367 },
    JumpStep::SetFlag { id: 281 },
    JumpStep::SetFlag { id: 282 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
    JumpStep::SetFlag { id: 98 },
    JumpStep::SetFlag { id: 284 },
    JumpStep::SetFlag { id: 285 },
    JumpStep::SetItem { id: 715, count: 1 },
    JumpStep::SetItem { id: 716, count: 1 },
    JumpStep::SetFlag { id: 283 },
    JumpStep::SetFlag { id: 100 },
    JumpStep::SetFlag { id: 286 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::SetFlag { id: 291 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::SetFlag { id: 290 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1405 },
    JumpStep::SetFlag { id: 287 },
    JumpStep::SetFlag { id: 292 },
    JumpStep::SetSnowFlag { id: 1051 },
    JumpStep::ClearFlag { id: 1405 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 288 },
    JumpStep::SetFlag { id: 301 },
    JumpStep::SetFlag { id: 1314 },
    JumpStep::SetItem { id: 1109, count: 1 },
    JumpStep::EquipWeapon { id: 1109 },
    JumpStep::SetItem { id: 250, count: 1 },
];

const JUNK_HEAP_LIFT_TO_FLOOR_B2_EXTRA_STEPS: [JumpStep; 21] = [
    JumpStep::SetSnowFlag { id: 5 },
    JumpStep::ClearFlag { id: 1314 },
    JumpStep::SetFlag { id: 302 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1406 },
    JumpStep::ClearFlag { id: 1406 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1404 },
    JumpStep::ClearFlag { id: 1404 },
    JumpStep::SetFlag { id: 1405 },
    JumpStep::ClearFlag { id: 1405 },
    JumpStep::SetFlag { id: 1406 },
    JumpStep::ClearFlag { id: 1406 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1407 },
    JumpStep::ClearFlag { id: 1407 },
];

const JUNK_HEAP_FLOOR_B2_TO_BOSS_FIGHT_5_EXTRA_STEPS: [JumpStep; 18] = [
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::SetFlag { id: 315 },
    JumpStep::SetTutorial { id: 2052 },
    JumpStep::SetFlag { id: 314 },
    JumpStep::SetTutorial { id: 2050 },
    JumpStep::SetFlag { id: 304 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::SetFlag { id: 313 },
    JumpStep::SetFlag { id: 303 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1404 },
    JumpStep::SetFlag { id: 305 },
    JumpStep::ClearFlag { id: 1404 },
    JumpStep::SetFlag { id: 1405 },
    JumpStep::ClearFlag { id: 1405 },
];

const BOSS_FIGHT_6_LEAD_IN_EXTRA_STEPS: [JumpStep; 16] = [
    JumpStep::SetFlag { id: 102 },
    JumpStep::SetFlag { id: 306 },
    JumpStep::SetFlag { id: 307 },
    JumpStep::SetTutorial { id: 2007 },
    JumpStep::SetFlag { id: 132 },
    JumpStep::LearnMagic { id: 8 },
    JumpStep::SetFlag { id: 308 },
    JumpStep::SetFlag { id: 309 },
    JumpStep::SetFlag { id: 310 },
    JumpStep::SetFlag { id: 374 },
    JumpStep::SetGameCounter { id: 100, value: 2 },
    JumpStep::SetGameCounter { id: 101, value: 1 },
    JumpStep::SetFlag { id: 311 },
    JumpStep::SetSnowFlag { id: 1056 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
];

const BOSS_FIGHT_6_TO_AFTER_EXTRA_STEPS: [JumpStep; 50] = [
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::SetFlag { id: 375 },
    JumpStep::SetFlag { id: 376 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1409 },
    JumpStep::SetFlag { id: 377 },
    JumpStep::SetFlag { id: 378 },
    JumpStep::SetFlag { id: 1307 },
    JumpStep::ClearFlag { id: 1409 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1404 },
    JumpStep::ClearFlag { id: 1404 },
    JumpStep::SetFlag { id: 1410 },
    JumpStep::ClearFlag { id: 1307 },
    JumpStep::SetFlag { id: 379 },
    JumpStep::ClearFlag { id: 1410 },
    JumpStep::SetFlag { id: 1405 },
    JumpStep::SetFlag { id: 380 },
    JumpStep::SetFlag { id: 381 },
    JumpStep::SetFlag { id: 1308 },
    JumpStep::ClearFlag { id: 1308 },
    JumpStep::SetFlag { id: 382 },
    JumpStep::SetFlag { id: 384 },
    JumpStep::SetFlag { id: 1309 },
    JumpStep::ClearFlag { id: 1405 },
    JumpStep::SetFlag { id: 1406 },
    JumpStep::ClearFlag { id: 1406 },
    JumpStep::SetFlag { id: 1411 },
    JumpStep::ClearFlag { id: 1309 },
    JumpStep::SetFlag { id: 383 },
    JumpStep::SetFlag { id: 385 },
    JumpStep::SetFlag { id: 386 },
    JumpStep::SetFlag { id: 1310 },
    JumpStep::ClearFlag { id: 1411 },
    JumpStep::SetFlag { id: 1410 },
    JumpStep::ClearFlag { id: 1410 },
    JumpStep::SetFlag { id: 1404 },
    JumpStep::ClearFlag { id: 1404 },
    JumpStep::SetFlag { id: 1403 },
    JumpStep::ClearFlag { id: 1310 },
    JumpStep::SetFlag { id: 125 },
    JumpStep::SetFlag { id: 128 },
    JumpStep::LearnMagic { id: 4 },
    JumpStep::SetTutorial { id: 2003 },
    JumpStep::SetFlag { id: 387 },
    JumpStep::ClearFlag { id: 1403 },
    JumpStep::SetFlag { id: 1409 },
];

const AFTER_BOSS_FIGHT_6_TO_FACADE_ENTRANCE_EXTRA_STEPS: [JumpStep; 12] = [
    JumpStep::ClearFlag { id: 1409 },
    JumpStep::SetFlag { id: 1402 },
    JumpStep::ClearFlag { id: 1402 },
    JumpStep::SetFlag { id: 1401 },
    JumpStep::ClearFlag { id: 1401 },
    JumpStep::SetGameCounter { id: 100, value: 3 },
    JumpStep::SetGameCounter { id: 101, value: 3 },
    JumpStep::SetSnowFlag { id: 1084 },
    JumpStep::SetFlag { id: 389 },
    JumpStep::SetFlag { id: 1364 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
];

const FACADE_ENTRANCE_TO_AFTER_FACADE_EXTRA_STEPS: [JumpStep; 21] = [
    JumpStep::SetFlag { id: 406 },
    JumpStep::SetFlag { id: 407 },
    JumpStep::SetFlag { id: 405 },
    JumpStep::SetFlag { id: 87 },
    JumpStep::SetFlag { id: 408 },
    JumpStep::SetFlag { id: 410 },
    JumpStep::SetFlag { id: 409 },
    JumpStep::SetFlag { id: 413 },
    JumpStep::SetFlag { id: 414 },
    JumpStep::SetFlag { id: 420 },
    JumpStep::SetFlag { id: 415 },
    JumpStep::SetFlag { id: 416 },
    JumpStep::SetFlag { id: 417 },
    JumpStep::SetFlag { id: 418 },
    JumpStep::SetFlag { id: 419 },
    JumpStep::SetFlag { id: 421 },
    JumpStep::SetFlag { id: 422 },
    JumpStep::SetFlag { id: 431 },
    JumpStep::SetFlag { id: 432 },
    JumpStep::SetFlag { id: 433 },
    JumpStep::SetFlag { id: 1325 },
];

const AFTER_FACADE_TO_TEMPLE_ENTRANCE_EXTRA_STEPS: [JumpStep; 6] = [
    JumpStep::SetFlag { id: 457 },
    JumpStep::SetFlag { id: 458 },
    JumpStep::SetFlag { id: 459 },
    JumpStep::SetFlag { id: 460 },
    JumpStep::SetFlag { id: 1436 },
    JumpStep::SetFlag { id: 434 },
];

const TEMPLE_ENTRANCE_TO_BEFORE_BOSS_EXTRA_STEPS: [JumpStep; 14] = [
    JumpStep::ClearFlag { id: 1325 },
    JumpStep::ClearFlag { id: 1436 },
    JumpStep::SetFlag { id: 89 },
    JumpStep::SetFlag { id: 435 },
    JumpStep::SetFlag { id: 448 },
    JumpStep::SetFlag { id: 436 },
    JumpStep::SetFlag { id: 449 },
    JumpStep::SetFlag { id: 437 },
    JumpStep::SetFlag { id: 439 },
    JumpStep::SetFlag { id: 450 },
    JumpStep::SetFlag { id: 451 },
    JumpStep::SetFlag { id: 452 },
    JumpStep::SetFlag { id: 453 },
    JumpStep::SetFlag { id: 454 },
];

const TEMPLE_BEFORE_BOSS_TO_VILLAGE_EXTRA_STEPS: [JumpStep; 15] = [
    JumpStep::SetFlag { id: 440 },
    JumpStep::SetFlag { id: 441 },
    JumpStep::SetFlag { id: 442 },
    JumpStep::SetFlag { id: 443 },
    JumpStep::SetFlag { id: 455 },
    JumpStep::SetFlag { id: 444 },
    JumpStep::SetItem { id: 1006, count: 1 },
    JumpStep::SetFlag { id: 133 },
    JumpStep::LearnMagic { id: 9 },
    JumpStep::SetTutorial { id: 2008 },
    JumpStep::SetFlag { id: 445 },
    JumpStep::SetSnowFlag { id: 1041 },
    JumpStep::SetFlag { id: 446 },
    JumpStep::SetGameCounter { id: 100, value: 3 },
    JumpStep::SetGameCounter { id: 101, value: 3 },
];

const VILLAGE_TO_SEAFRONT_WITH_RED_BAG_EXTRA_STEPS: [JumpStep; 24] = [
    JumpStep::SetFlag { id: 1365 },
    JumpStep::SetFlag { id: 470 },
    JumpStep::SetFlag { id: 471 },
    JumpStep::SetFlag { id: 472 },
    JumpStep::SetFlag { id: 473 },
    JumpStep::SetFlag { id: 254 },
    JumpStep::SetFlag { id: 93 },
    JumpStep::SetFlag { id: 134 },
    JumpStep::SetFlag { id: 475 },
    JumpStep::SetFlag { id: 476 },
    JumpStep::SetFlag { id: 558 },
    JumpStep::SetFlag { id: 559 },
    JumpStep::SetFlag { id: 560 },
    JumpStep::SetFlag { id: 477 },
    JumpStep::SetFlag { id: 1366 },
    JumpStep::SetGameCounter { id: 200, value: 1 },
    JumpStep::SetSnowFlag { id: 11 },
    JumpStep::SetSnowFlag { id: 12 },
    JumpStep::SetSnowFlag { id: 14 },
    JumpStep::SetSnowFlag { id: 15 },
    JumpStep::SetSnowFlag { id: 16 },
    JumpStep::SetSnowFlag { id: 26 },
    JumpStep::SetSnowFlag { id: 18 },
    JumpStep::SetItem { id: 325, count: 1 },
];

const SEAFRONT_WITH_RED_BAG_TO_MANOR_FRONT_EXTRA_STEPS: [JumpStep; 11] = [
    JumpStep::SetItem { id: 325, count: 0 },
    JumpStep::SetSnowFlag { id: 19 },
    JumpStep::SetSnowFlag { id: 20 },
    JumpStep::SetSnowFlag { id: 22 },
    JumpStep::SetSnowFlag { id: 21 },
    JumpStep::SetSnowFlag { id: 24 },
    JumpStep::SetFlag { id: 499 },
    JumpStep::SetFlag { id: 500 },
    JumpStep::SetFlag { id: 501 },
    JumpStep::SetFlag { id: 1311 },
    JumpStep::SetFlag { id: 348 },
];

const VILLAGE_TO_FOREST_INSIDE_EXTRA_STEPS: [JumpStep; 11] = [
    JumpStep::SetFlag { id: 1365 },
    JumpStep::SetFlag { id: 470 },
    JumpStep::SetFlag { id: 1355 },
    JumpStep::SetFlag { id: 1356 },
    JumpStep::SetFlag { id: 471 },
    JumpStep::SetFlag { id: 472 },
    JumpStep::SetFlag { id: 473 },
    JumpStep::SetGameCounter { id: 200, value: 1 },
    JumpStep::SetFlag { id: 254 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
];

const MANOR_FRONT_TO_EMIL_FOLLOWS_EXTRA_STEPS: [JumpStep; 27] = [
    JumpStep::ClearFlag { id: 1311 },
    JumpStep::SetFlag { id: 502 },
    JumpStep::SetFlag { id: 503 },
    JumpStep::SetFlag { id: 104 },
    JumpStep::SetFlag { id: 522 },
    JumpStep::SetFlag { id: 504 },
    JumpStep::SetFlag { id: 505 },
    JumpStep::SetFlag { id: 526 },
    JumpStep::SetFlag { id: 516 },
    JumpStep::SetFlag { id: 506 },
    JumpStep::SetFlag { id: 507 },
    JumpStep::SetFlag { id: 517 },
    JumpStep::SetFlag { id: 508 },
    JumpStep::SetFlag { id: 518 },
    JumpStep::SetFlag { id: 523 },
    JumpStep::SetFlag { id: 524 },
    JumpStep::SetFlag { id: 509 },
    JumpStep::SetFlag { id: 136 },
    JumpStep::SetFlag { id: 515 },
    JumpStep::SetSnowFlag { id: 1050 },
    JumpStep::SetItem { id: 717, count: 1 },
    JumpStep::SetItem { id: 256, count: 1 },
    JumpStep::SetItem { id: 257, count: 1 },
    JumpStep::SetItem { id: 258, count: 1 },
    JumpStep::SetItem { id: 259, count: 1 },
    JumpStep::SetGameCounter {
        id: 100,
        value: 255,
    },
    JumpStep::SetGameCounter {
        id: 101,
        value: 255,
    },
];

const EMIL_FOLLOWS_TO_BEFORE_BOSS_FIGHT_8_EXTRA_STEPS: [JumpStep; 3] = [
    JumpStep::SetFlag { id: 1433 },
    JumpStep::SetFlag { id: 1434 },
    JumpStep::SetFlag { id: 1435 },
];

const BEFORE_BOSS_FIGHT_8_TO_BOSS_FIGHT_9_EXTRA_STEPS: [JumpStep; 23] = [
    JumpStep::ClearFlag { id: 1433 },
    JumpStep::ClearFlag { id: 1434 },
    JumpStep::ClearFlag { id: 1435 },
    JumpStep::ClearFlag { id: 136 },
    JumpStep::SetFlag { id: 510 },
    JumpStep::SetFlag { id: 511 },
    JumpStep::SetFlag { id: 512 },
    JumpStep::SetFlag { id: 131 },
    JumpStep::SetFlag { id: 106 },
    JumpStep::SetFlag { id: 513 },
    JumpStep::SetFlag { id: 533 },
    JumpStep::SetFlag { id: 534 },
    JumpStep::SetFlag { id: 535 },
    JumpStep::SetFlag { id: 536 },
    JumpStep::SetFlag { id: 537 },
    JumpStep::SetFlag { id: 538 },
    JumpStep::SetFlag { id: 539 },
    JumpStep::SetFlag { id: 540 },
    JumpStep::SetFlag { id: 1367 },
    JumpStep::SetSnowFlag { id: 1065 },
    JumpStep::SetItem { id: 267, count: 1 },
    JumpStep::SetGameCounter { id: 100, value: 3 },
    JumpStep::SetGameCounter { id: 101, value: 3 },
];

const BOSS_FIGHT_9_TO_BOSS_FIGHT_10_EXTRA_STEPS: [JumpStep; 20] = [
    JumpStep::SetFlag { id: 541 },
    JumpStep::SetFlag { id: 542 },
    JumpStep::SetFlag { id: 543 },
    JumpStep::SetFlag { id: 544 },
    JumpStep::SetFlag { id: 545 },
    JumpStep::SetFlag { id: 546 },
    JumpStep::SetFlag { id: 547 },
    JumpStep::SetFlag { id: 548 },
    JumpStep::SetFlag { id: 549 },
    JumpStep::ClearFlag { id: 550 },
    JumpStep::ClearFlag { id: 551 },
    JumpStep::ClearFlag { id: 552 },
    JumpStep::ClearFlag { id: 553 },
    JumpStep::ClearFlag { id: 554 },
    JumpStep::ClearFlag { id: 555 },
    JumpStep::ClearFlag { id: 556 },
    JumpStep::ClearFlag { id: 1298 },
    JumpStep::ClearFlag { id: 1330 },
    JumpStep::LearnMagic { id: 7 },
    JumpStep::LearnMagic { id: 10 },
];

const BOSS_FIGHT_10_TO_BOSS_FIGHT_11_EXTRA_STEPS: [JumpStep; 3] = [
    JumpStep::SetFlag { id: 550 },
    JumpStep::SetFlag { id: 551 },
    JumpStep::SetFlag { id: 552 },
];

const AFTER_PROLOGUE_MP_STEP: JumpStep = JumpStep::MpFullMode { on: false };
const AFTER_PROLOGUE_PHASE: &str = "A_CENTER_VILLAGE_01";
const AFTER_PROLOGUE_POINT: i32 = 8;
const POPOLA_TALK_PHASE: &str = "A_CENTER_LIBRARY_01";
const POPOLA_TALK_POINT: i32 = 1;
const BEFORE_NEXT_DAY_PHASE: &str = "A_CENTER_VILLAGE_01";
const BEFORE_NEXT_DAY_POINT: i32 = 4;
const AT_SHRINE_ENTRANCE_PHASE: &str = "A_STONE_TEMPLE_01";
const AT_SHRINE_ENTRANCE_POINT: i32 = 1;
const AT_SHRINE_TOP_PHASE: &str = "A_STONE_FIELD_01";
const AT_SHRINE_TOP_POINT: i32 = 3;
const AT_SHRINE_BOSS_PHASE: &str = "A_STONE_FIELD_02";
const AT_SHRINE_BOSS_POINT: i32 = 1;
const BOSS_FIGHT_3_PHASE: &str = "A_CLIFF_VILLAGE_01";
const BOSS_FIGHT_3_POINT: i32 = 1;
const JUNK_HEAP_FLOOR_B2_PHASE: &str = "A_MOUNT_ROBOT_02";
const JUNK_HEAP_FLOOR_B2_POINT: i32 = 1;
const JUNK_HEAP_BOSS_FIGHT_5_PHASE: &str = "A_MOUNT_ROBOT_10";
const JUNK_HEAP_BOSS_FIGHT_5_POINT: i32 = 1;
const BOSS_FIGHT_6_PHASE: &str = "A_CLIFF_VILLAGE_01";
const BOSS_FIGHT_6_POINT: i32 = 1;
const AFTER_BOSS_FIGHT_6_PHASE: &str = "A_CLIFF_VILLAGE_01";
const AFTER_BOSS_FIGHT_6_POINT: i32 = 1;
const AFTER_FACADE_PHASE: &str = "A_DESERT_FIELD_01";
const AFTER_FACADE_POINT: i32 = 3;
const TEMPLE_ENTRANCE_PHASE: &str = "A_DESERT_TEMPLE_01";
const TEMPLE_ENTRANCE_POINT: i32 = 1;
const TEMPLE_BEFORE_BOSS_PHASE: &str = "A_DESERT_TEMPLE_02";
const TEMPLE_BEFORE_BOSS_POINT: i32 = 1;
const VILLAGE_BACK_FROM_TEMPLE_PHASE: &str = "A_CENTER_VILLAGE_01";
const VILLAGE_BACK_FROM_TEMPLE_POINT: i32 = 2;
const SEAFRONT_WITH_RED_BAG_PHASE: &str = "A_SOUTH_FIELD_01";
const SEAFRONT_WITH_RED_BAG_POINT: i32 = 3;
const MANOR_FRONT_PHASE: &str = "A_SOUTH_FIELD_01";
const MANOR_FRONT_POINT: i32 = 2;
const MANOR_EMIL_FOLLOWS_PHASE: &str = "A_SOUTH_MANSION_01";
const MANOR_EMIL_FOLLOWS_POINT: i32 = 1;
const MANOR_BEFORE_BOSS_FIGHT_8_PHASE: &str = "A_SOUTH_MANSION_02";
const MANOR_BEFORE_BOSS_FIGHT_8_POINT: i32 = 1;
const BOSS_FIGHT_9_PHASE: &str = "A_CENTER_VILLAGE_01";
const BOSS_FIGHT_9_POINT: i32 = 4;
const BOSS_FIGHT_10_PHASE: &str = "A_CENTER_LIBRARY_01";
const BOSS_FIGHT_10_POINT: i32 = 1;
const BOSS_FIGHT_11_PHASE: &str = "A_CENTER_LIBRARY_01";
const BOSS_FIGHT_11_POINT: i32 = 1;
const FOREST_INSIDE_PHASE: &str = "C_FOREST_FIELD_01";
const FOREST_INSIDE_POINT: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpStep {
    SetFlag { id: u16 },
    ClearFlag { id: u16 },
    SetTutorial { id: u16 },
    SetSnowFlag { id: u16 },
    SetGameCounter { id: u8, value: u8 },
    SetItem { id: i32, count: i32 },
    EquipWeapon { id: i32 },
    MpFullMode { on: bool },
    SetHaveWhiteBook { on: bool },
    LearnMagic { id: i32 },
    SetMagicSlot { slot: i32, magic: i32 },
}

pub fn loadout_steps(point: StoryJumpPoint) -> Vec<JumpStep> {
    match point {
        StoryJumpPoint::VillageAfterPrologue => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::LibraryPopolaTalk => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.extend_from_slice(&POPOLA_TALK_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::VillageBeforeNextDay => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.extend_from_slice(&BEFORE_NEXT_DAY_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ShrineEntrance => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.extend_from_slice(&BEFORE_NEXT_DAY_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_ENTRANCE_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ShrineTop => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.extend_from_slice(&BEFORE_NEXT_DAY_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_ENTRANCE_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_TOP_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ShrineBoss2 => {
            let mut steps = AFTER_PROLOGUE_STEPS.to_vec();
            steps.extend_from_slice(&BEFORE_NEXT_DAY_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_ENTRANCE_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_TOP_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_BOSS_EXTRA_STEPS);
            steps.extend_from_slice(&AT_SHRINE_ARENA_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::AerieBoss3 => {
            let mut steps = loadout_steps(StoryJumpPoint::ShrineBoss2);
            steps.pop();
            steps.extend_from_slice(&AERIE_ENTRANCE_EXTRA_STEPS);
            steps.extend_from_slice(&BOSS_FIGHT_3_LEAD_IN_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::JunkHeapB2 => {
            let mut steps = loadout_steps(StoryJumpPoint::AerieBoss3);
            steps.pop();
            steps.extend_from_slice(&BOSS_FIGHT_3_TO_JUNK_HEAP_FIRST_UPGRADE_EXTRA_STEPS);
            steps.extend_from_slice(&JUNK_HEAP_LIFT_TO_FLOOR_B2_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::JunkHeapBoss5 => {
            let mut steps = loadout_steps(StoryJumpPoint::JunkHeapB2);
            steps.pop();
            steps.extend_from_slice(&JUNK_HEAP_FLOOR_B2_TO_BOSS_FIGHT_5_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::AerieBoss6 => {
            let mut steps = loadout_steps(StoryJumpPoint::JunkHeapBoss5);
            steps.pop();
            steps.extend_from_slice(&BOSS_FIGHT_6_LEAD_IN_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::AerieAfterBoss6 => {
            let mut steps = loadout_steps(StoryJumpPoint::AerieBoss6);
            steps.pop();
            steps.extend_from_slice(&BOSS_FIGHT_6_TO_AFTER_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::DesertAfterFacade => {
            let mut steps = loadout_steps(StoryJumpPoint::AerieAfterBoss6);
            steps.pop();
            steps.extend_from_slice(&AFTER_BOSS_FIGHT_6_TO_FACADE_ENTRANCE_EXTRA_STEPS);
            steps.extend_from_slice(&FACADE_ENTRANCE_TO_AFTER_FACADE_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::TempleEntrance => {
            let mut steps = loadout_steps(StoryJumpPoint::DesertAfterFacade);
            steps.pop();
            steps.extend_from_slice(&AFTER_FACADE_TO_TEMPLE_ENTRANCE_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::TempleBoss7 => {
            let mut steps = loadout_steps(StoryJumpPoint::TempleEntrance);
            steps.pop();
            steps.extend_from_slice(&TEMPLE_ENTRANCE_TO_BEFORE_BOSS_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::VillageAfterTemple => {
            let mut steps = loadout_steps(StoryJumpPoint::TempleBoss7);
            steps.pop();
            steps.extend_from_slice(&TEMPLE_BEFORE_BOSS_TO_VILLAGE_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::SouthPlainsRedBag => {
            let mut steps = loadout_steps(StoryJumpPoint::VillageAfterTemple);
            steps.pop();
            steps.extend_from_slice(&VILLAGE_TO_SEAFRONT_WITH_RED_BAG_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ManorFront => {
            let mut steps = loadout_steps(StoryJumpPoint::SouthPlainsRedBag);
            steps.pop();
            steps.extend_from_slice(&SEAFRONT_WITH_RED_BAG_TO_MANOR_FRONT_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ForestArrival => {
            let mut steps = loadout_steps(StoryJumpPoint::VillageAfterTemple);
            steps.pop();
            steps.extend_from_slice(&VILLAGE_TO_FOREST_INSIDE_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ManorEmilFollows => {
            let mut steps = loadout_steps(StoryJumpPoint::ManorFront);
            steps.pop();
            steps.extend_from_slice(&MANOR_FRONT_TO_EMIL_FOLLOWS_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::ManorBoss8 => {
            let mut steps = loadout_steps(StoryJumpPoint::ManorEmilFollows);
            steps.pop();
            steps.extend_from_slice(&EMIL_FOLLOWS_TO_BEFORE_BOSS_FIGHT_8_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::VillageBoss9 => {
            let mut steps = loadout_steps(StoryJumpPoint::ManorBoss8);
            steps.pop();
            steps.extend_from_slice(&BEFORE_BOSS_FIGHT_8_TO_BOSS_FIGHT_9_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::LibraryBoss10 => {
            let mut steps = loadout_steps(StoryJumpPoint::VillageBoss9);
            steps.pop();
            steps.extend_from_slice(&BOSS_FIGHT_9_TO_BOSS_FIGHT_10_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
        StoryJumpPoint::LibraryBoss11 => {
            let mut steps = loadout_steps(StoryJumpPoint::LibraryBoss10);
            steps.pop();
            steps.extend_from_slice(&BOSS_FIGHT_10_TO_BOSS_FIGHT_11_EXTRA_STEPS);
            steps.push(AFTER_PROLOGUE_MP_STEP);
            steps
        }
    }
}

pub fn destination(point: StoryJumpPoint) -> (&'static str, i32) {
    match point {
        StoryJumpPoint::VillageAfterPrologue => (AFTER_PROLOGUE_PHASE, AFTER_PROLOGUE_POINT),
        StoryJumpPoint::LibraryPopolaTalk => (POPOLA_TALK_PHASE, POPOLA_TALK_POINT),
        StoryJumpPoint::VillageBeforeNextDay => (BEFORE_NEXT_DAY_PHASE, BEFORE_NEXT_DAY_POINT),
        StoryJumpPoint::ShrineEntrance => (AT_SHRINE_ENTRANCE_PHASE, AT_SHRINE_ENTRANCE_POINT),
        StoryJumpPoint::ShrineTop => (AT_SHRINE_TOP_PHASE, AT_SHRINE_TOP_POINT),
        StoryJumpPoint::ShrineBoss2 => (AT_SHRINE_BOSS_PHASE, AT_SHRINE_BOSS_POINT),
        StoryJumpPoint::AerieBoss3 => (BOSS_FIGHT_3_PHASE, BOSS_FIGHT_3_POINT),
        StoryJumpPoint::JunkHeapB2 => (JUNK_HEAP_FLOOR_B2_PHASE, JUNK_HEAP_FLOOR_B2_POINT),
        StoryJumpPoint::JunkHeapBoss5 => {
            (JUNK_HEAP_BOSS_FIGHT_5_PHASE, JUNK_HEAP_BOSS_FIGHT_5_POINT)
        }
        StoryJumpPoint::AerieAfterBoss6 => (AFTER_BOSS_FIGHT_6_PHASE, AFTER_BOSS_FIGHT_6_POINT),
        StoryJumpPoint::AerieBoss6 => (BOSS_FIGHT_6_PHASE, BOSS_FIGHT_6_POINT),
        StoryJumpPoint::DesertAfterFacade => (AFTER_FACADE_PHASE, AFTER_FACADE_POINT),
        StoryJumpPoint::TempleEntrance => (TEMPLE_ENTRANCE_PHASE, TEMPLE_ENTRANCE_POINT),
        StoryJumpPoint::TempleBoss7 => (TEMPLE_BEFORE_BOSS_PHASE, TEMPLE_BEFORE_BOSS_POINT),
        StoryJumpPoint::VillageAfterTemple => (
            VILLAGE_BACK_FROM_TEMPLE_PHASE,
            VILLAGE_BACK_FROM_TEMPLE_POINT,
        ),
        StoryJumpPoint::SouthPlainsRedBag => {
            (SEAFRONT_WITH_RED_BAG_PHASE, SEAFRONT_WITH_RED_BAG_POINT)
        }
        StoryJumpPoint::ManorFront => (MANOR_FRONT_PHASE, MANOR_FRONT_POINT),
        StoryJumpPoint::ForestArrival => (FOREST_INSIDE_PHASE, FOREST_INSIDE_POINT),
        StoryJumpPoint::ManorEmilFollows => (MANOR_EMIL_FOLLOWS_PHASE, MANOR_EMIL_FOLLOWS_POINT),
        StoryJumpPoint::ManorBoss8 => (
            MANOR_BEFORE_BOSS_FIGHT_8_PHASE,
            MANOR_BEFORE_BOSS_FIGHT_8_POINT,
        ),
        StoryJumpPoint::VillageBoss9 => (BOSS_FIGHT_9_PHASE, BOSS_FIGHT_9_POINT),
        StoryJumpPoint::LibraryBoss10 => (BOSS_FIGHT_10_PHASE, BOSS_FIGHT_10_POINT),
        StoryJumpPoint::LibraryBoss11 => (BOSS_FIGHT_11_PHASE, BOSS_FIGHT_11_POINT),
    }
}
