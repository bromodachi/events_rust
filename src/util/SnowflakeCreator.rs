use std::cmp::{max, min};
use std::sync::{Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SnowflakeCreator {
    // any number that's within 5 bits
    worker_id: u64,
    // should be the pod id.
    process_id: u64,
    // when the snowflake started creating in milliseconds
    start_millis: u64,
    // 12 bit number
    increment: Mutex<u16>,
}

impl SnowflakeCreator {

    pub fn new(
        worker_id: u64,
        process_id: u64,
        start_millis: u64,
        start_counter: u16
    ) ->SnowflakeCreator {
        Self {
            worker_id: worker_id,
            process_id: process_id,
            start_millis,
            increment: Mutex::new(start_counter)
        }
    }
    pub fn create_id(&self, timestamp: u64) -> u64 {
        // TODO: Consider making this a result instead to do some error checking
        let first_second: u64 = self.start_millis;
        let timestamp = (timestamp - first_second) << 22;
        let worker_id = self.worker_id << 17;
        let process_id = self.process_id << 12;
        let mut increment = self.increment.lock().unwrap();
        *increment += 1;
        // to not overflow
        *increment = *increment % 4096;
        let value =  timestamp | worker_id | process_id | *increment as u64;
        value
    }

    pub fn get_time(&self, snowflake: u64) -> u64 {
        (snowflake >> 22)  + self.start_millis
    }

    pub fn convert_time_to_snowflake(&self, time_millis: &u64) -> u64 {
        // we want the number to be kept within a 41 bit number.
        (max(min(*time_millis, 2_u64.pow(41) + (self.start_millis - 1)), self.start_millis) - self.start_millis) << 22
    }

    pub fn get_increment(snowflake: &u64) -> u64 {
        snowflake & 0xFFF
    }

    pub fn get_timestamp(&self, snowflake: &u64) -> u64 {
        (snowflake >> 22) - self.start_millis
    }
}