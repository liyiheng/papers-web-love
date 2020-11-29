/// An O(1) algorithm for implementing the LFU cache eviction scheme
/// by Prof. Ketan Shah, Anirban Mitra, Dhruv Matani
/// [Paper](https://github.com/papers-we-love/papers-we-love/blob/master/caching/a-constant-algorithm-for-implementing-the-lfu-cache-eviction-scheme.pdf)
pub mod lfu;

/// 2Q: A Low Overhead High Performance Buffer Management Replacement Algorithm
/// by Theodore Johnson and Dennis Shasha
/// [Paper](http://www.vldb.org/conf/1994/P439.PDF)
pub mod two_q_lru;
