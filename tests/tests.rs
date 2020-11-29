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
mod lru_two_q {
    use papers_web_love::caching::two_q_lru::SimplifiedTwoQ;
    #[test]
    fn basic() {
        let mut cache = SimplifiedTwoQ::with_threshold(10, 5);
        for i in 0..5 {
            cache.insert(i, i);
            assert_eq!(cache.get(&i), Some(&i));
        }
        // Am [4,3,2,1,0]
        // A1 []

        for i in 5..10 {
            cache.insert(i, i);
            assert_eq!(cache.get(&i), Some(&i));
        }
        // Am [9,8,7,6,5]
        // A1 []

        for i in 10..15 {
            cache.insert(i, i);
        }
        // Am [9,8,7,6,5]
        // A1 [14,13,12,11,10]

        cache.insert(15, 15);
        assert!(cache.get(&10).is_none());
        // Am [9,8,7,6,5]
        // A1 [15,14,13,12,11] (10 dropped)

        for i in 11..=15 {
            assert_eq!(cache.get(&i), Some(&i));
        }
        // Am [15, 14, 13, 12, 11, 9, 8, 7, 6, 5]
        // A1 []

        for i in 5..=15 {
            if i == 10 {
                continue;
            }
            assert_eq!(cache.get(&i), Some(&i));
        }
        // Am [15, 14, 13, 12, 11, 9, 8, 7, 6, 5]
        // A1 []

        cache.insert(10, 10); // Push 10 to A1, 5 dropped.
                              // Am [15, 14, 13, 12, 11, 9, 8, 7, 6]
                              // A1 [10]
        assert!(cache.get(&5).is_none());
    }

    #[test]
    fn robust() {
        let mut cache = SimplifiedTwoQ::with_threshold(10, 5);
        for i in 0..10000 {
            cache.insert(i, i);
            if i % 3 == 0 {
                cache.get(&i);
            }
            if i % 100 == 0 {
                cache.clear();
            }
        }
    }
}
