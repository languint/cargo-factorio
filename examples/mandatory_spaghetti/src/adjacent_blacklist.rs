const BLACKLIST: &[&str] = &[
    "curved-rail-a",
    "curved-rail-b",
    "straight-rail",
    "rail-ramp",
    "rail-support",
    "half-diagonal-rail",
    "elevated-curved-rail-a",
    "elevated-curved-rail-b",
    "elevated-straight-rail",
    "elevated-half-diagonal-rail",
    "legacy-curved-rail",
    "legacy-straight-rail",
    "rail-signal",
    "rail-chain-signal",
    "train-stop",
    "locomotive",
    "cargo-wagon",
    "fluid-wagon",
    "artillery-wagon",
    "car",
    "tank",
    "artillery-turret",
    "cargo-landing-pad",
];

pub fn check(entity_type: &str) -> bool {
    for item in BLACKLIST {
        if *item == entity_type {
            return true;
        }
    }
    false
}
