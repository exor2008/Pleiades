use core::cmp::{max, min};

pub struct CooldownValue<const COOLDOWNL: u8, const MIN: usize, const MAX: usize> {
    value: usize,
    cooldown: u8,
}

impl<const COOLDOWNL: u8, const MIN: usize, const MAX: usize> CooldownValue<COOLDOWNL, MIN, MAX> {
    pub fn new(value: usize) -> Self {
        CooldownValue { value, cooldown: 0 }
    }

    pub fn up(&mut self) {
        match self.cooldown == 0 {
            true => {
                self.cooldown = COOLDOWNL;
                self.value += 1;
                self.value = min(self.value, MAX);
            }
            false => {
                self.cooldown -= 1;
            }
        }
    }

    pub fn down(&mut self) {
        match self.cooldown == 0 {
            true => {
                self.cooldown = COOLDOWNL;
                self.value -= 1;
                self.value = max(self.value, MIN);
            }
            false => {
                self.cooldown -= 1;
            }
        }
    }

    pub fn value(&self) -> &usize {
        &self.value
    }
}
