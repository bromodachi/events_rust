
#[cfg(test)]
mod tests {
    use events::util::SnowflakeCreator::SnowflakeCreator;
    #[test]
    fn convert_to_snow_flake() {
        let mut test = SnowflakeCreator::new(1, 0, 1420070400000, 6);
        let snowflake = test.create_id(1462015105796);
        assert_eq!(175928847299117063, snowflake)
    }

    #[test]
    fn check_increment_does_not_overflow() {
        let mut  test = SnowflakeCreator::new(1, 0, 1420070400000, 4095);
        let snowflake = test.create_id(1462015105796);

        assert_eq!(0, SnowflakeCreator::get_increment(&snowflake))
    }

    #[test]
    fn check_max_number() {
        let mut  test = SnowflakeCreator::new(1, 0, 1420070400000, 4095);
        // Monday, September 7, 2093. Most likely, we won't be alive until then~
        let snowflake = test.convert_time_to_snowflake(&3_903_090_455_554);
        // our max number for this particular start_millis
        assert_eq!(snowflake, 9223372036850581504)
    }

    #[test]
    fn july_27th_to_snowflake()  {
        let snowflake_creator = SnowflakeCreator::new(1, 0, 1420070400000, 0);
        // 1722006000000 - July 27th 00:00:00 JST.
        // 1722092399000- July 27th 23:59:59 JST.
        let snowflake = snowflake_creator.create_id(1722006000000);
        assert_eq!(1266409694822531073, snowflake);
        assert_eq!(1266409694822400000, snowflake_creator.convert_time_to_snowflake(&1722006000000));
        assert_eq!(1266772078493696000, snowflake_creator.convert_time_to_snowflake(&1722092399000));
        // (60 * 60 * 24)  * 1000 - 1000; -> 60 seconds, 60 minutes, 24 hours. Minus 1 ms as we don't include ms.
        // 86,399,000
        assert_eq!((1266772078493696000 - 1266409694822400000) >> 22, ((60 * 60 * 24)  * 1000 - 1000) as u64);
    }
}