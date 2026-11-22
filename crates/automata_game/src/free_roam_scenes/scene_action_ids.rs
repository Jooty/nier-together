use crate::free_roam_scenes::HapWalkCap;

pub const DENIED_SCENE_ACTION_IDS: [u32; 132] = [
    0x01977575, 0x05BD467D, 0x076E520C, 0x0A6A9722, 0x0B958A0B, 0x0E3F914C, 0x0FC1BBF3, 0x0FC2DB38,
    0x0FDC2321, 0x108D1ABF, 0x111F87D8, 0x127EF5ED, 0x13BA6C0A, 0x14006B34, 0x15973503, 0x15BD1E83,
    0x16002123, 0x16D730BD, 0x17AF2083, 0x187B1028, 0x1B273A24, 0x1F4627D0, 0x22B119D6, 0x22B270BF,
    0x2386F571, 0x2A186CB8, 0x2DA0A3B5, 0x2DF117FE, 0x2ECE0D3B, 0x32AF803F, 0x33492B64, 0x35E8BED3,
    0x35FAD482, 0x3C901B05, 0x3D1B8F16, 0x3E7089E1, 0x3E944C1F, 0x3F27D712, 0x3F390565, 0x40752126,
    0x40F306C0, 0x41C15C7E, 0x46C80158, 0x4AE9D8E4, 0x4BD13CE6, 0x4FBFB215, 0x50CCA8FC, 0x51AFCF84,
    0x55018D06, 0x55AB8FD0, 0x55E0D5C3, 0x56F2ABD7, 0x5A3386B6, 0x5A798BB0, 0x5A9C11C7, 0x5BE1A049,
    0x5C745714, 0x5E7AF572, 0x5EDBC752, 0x61CC136C, 0x63BF1A7D, 0x67E9EA80, 0x687746DF, 0x6B7B2C2D,
    0x6CAF0544, 0x6DA3117B, 0x730114ED, 0x7513207C, 0x78159020, 0x7883C00D, 0x79A557DF, 0x7A648611,
    0x7DC0F895, 0x7F1B6A60, 0x7F81ECE5, 0x8290DF68, 0x885A6D15, 0x8AED5DB2, 0x8CFB0D91, 0x8F510FD7,
    0x91A70ECC, 0x91CAC295, 0x9428C4ED, 0x965418D4, 0x96C0A342, 0x981416F8, 0x9D1ECF12, 0x9D7B5CC4,
    0x9E78F457, 0x9E86F7D8, 0x9EDF12AC, 0xA04F7AB5, 0xA0CB8ADE, 0xA5C9CFFF, 0xA9703D94, 0xAC94CD05,
    0xAE3428D5, 0xAEC4696B, 0xB1003A4E, 0xB3396648, 0xB34E2FE6, 0xB38C48D4, 0xB58233A5, 0xB8A361ED,
    0xB8EC61F8, 0xBA399355, 0xBBEB7C70, 0xBFF411D1, 0xC205C81C, 0xC6115145, 0xC936917D, 0xCD0DC410,
    0xD54269FF, 0xD63C740D, 0xD9279D5B, 0xD980A0BF, 0xDA68452B, 0xDAADEE66, 0xDDE87DD1, 0xE35E6B75,
    0xE767F923, 0xEA170E37, 0xEC14ECF5, 0xEF5A44E2, 0xEF67F5AA, 0xF03C3B85, 0xF2E294A4, 0xF4BE212E,
    0xF695F59A, 0xF8FDC3A8, 0xF98DF8F5, 0xF9DBA7C9,
];

pub const HAP_WALK_CAPS: [HapWalkCap; 27] = [
    HapWalkCap {
        index: 5,
        action_id: 0x4084C15C,
    },
    HapWalkCap {
        index: 10,
        action_id: 0x46CD6E49,
    },
    HapWalkCap {
        index: 8,
        action_id: 0x4FD37BF9,
    },
    HapWalkCap {
        index: 3,
        action_id: 0x50465D1D,
    },
    HapWalkCap {
        index: 5,
        action_id: 0x51AFCF84,
    },
    HapWalkCap {
        index: 2,
        action_id: 0x5BE1A049,
    },
    HapWalkCap {
        index: 19,
        action_id: 0x7CEF1D21,
    },
    HapWalkCap {
        index: 17,
        action_id: 0x7D9E2B81,
    },
    HapWalkCap {
        index: 5,
        action_id: 0x82745AE8,
    },
    HapWalkCap {
        index: 1,
        action_id: 0x848664B3,
    },
    HapWalkCap {
        index: 0,
        action_id: 0x85D06139,
    },
    HapWalkCap {
        index: 20,
        action_id: 0x880C00CA,
    },
    HapWalkCap {
        index: 8,
        action_id: 0x88B4A7C5,
    },
    HapWalkCap {
        index: 16,
        action_id: 0x8D9E0D7A,
    },
    HapWalkCap {
        index: 12,
        action_id: 0x91A70F6C,
    },
    HapWalkCap {
        index: 15,
        action_id: 0x97FA8FF9,
    },
    HapWalkCap {
        index: 3,
        action_id: 0xA2D65230,
    },
    HapWalkCap {
        index: 1,
        action_id: 0xBA3D3AE9,
    },
    HapWalkCap {
        index: 3,
        action_id: 0xBEF34B8B,
    },
    HapWalkCap {
        index: 4,
        action_id: 0xCA4B080B,
    },
    HapWalkCap {
        index: 2,
        action_id: 0xCDC8E234,
    },
    HapWalkCap {
        index: 4,
        action_id: 0xD94DED21,
    },
    HapWalkCap {
        index: 5,
        action_id: 0xDC4CFB8C,
    },
    HapWalkCap {
        index: 0,
        action_id: 0xEF770563,
    },
    HapWalkCap {
        index: 1,
        action_id: 0xF03C3B85,
    },
    HapWalkCap {
        index: 3,
        action_id: 0xF0B3105C,
    },
    HapWalkCap {
        index: 0,
        action_id: 0xF46EAF71,
    },
];

pub const FREE_ROAM_NEEDED_SCENE_ACTION_IDS: [u32; 2] = [0xA3EF297A, 0xCEDF793A];
