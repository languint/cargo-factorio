const BLACKLIST: &[&str] = &[
    "electric-pole",
    "small-electric-pole",
    "medium-electric-pole",
    "big-electric-pole",
    "substation",
    "pipe",
    "pipe-to-ground",
    "pump",
    "offshore-pump",
    "storage-tank",
    "rail-signal",
    "rail-chain-signal",
    "train-stop",
    "roboport",
    "lamp",
    "power-switch",
    "programmable-speaker",
    "land-mine",
    "gate",
    "wall",
];

pub fn check(entity_type: &str) -> bool {
    for item in BLACKLIST {
        if *item == entity_type {
            return true;
        }
    }
    false
}
