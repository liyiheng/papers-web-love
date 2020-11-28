mod lfu {
    use papers_web_love::caching::lfu::LfuCache;
    #[test]
    fn robust() {
        let capacity = 10;
        let mut cache = LfuCache::new(capacity);
        for i in 0..capacity * 100 {
            cache.insert(i, i);
        }
        for i in 0..capacity * 100 {
            cache.insert(i % 3, i);
            cache.insert(i % 5, i);
        }
        for _ in 0..100 {
            for i in 0..capacity {
                cache.get(&i);
            }
        }
    }

    #[test]
    fn basic() {
        let capacity = 10;
        let mut cache = LfuCache::new(capacity);
        for i in 0..capacity {
            cache.insert(i, i);
            assert_eq!(cache.get(&i), Some(&i));
        }
        for i in 0..capacity {
            assert_eq!(cache.get(&i), Some(&i));
        }
        assert_eq!(cache.get(&5), Some(&5));
        assert_eq!(cache.get(&5), Some(&5));

        assert_eq!(cache.get(&5), Some(&5));
        assert_eq!(cache.get(&6), Some(&6));
        assert_eq!(cache.get(&6), Some(&6));
        for i in 2..capacity {
            cache.insert(i, i);
        }
        assert_eq!(cache.get(&5), Some(&5));
        assert_eq!(cache.get(&6), Some(&6));
    }
}
