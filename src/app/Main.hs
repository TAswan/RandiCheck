module Main where

import Test.QuickCheck


-- check that addition is associative with an additional predicate on inputs such that a + b == c
main :: IO ()
main = do                       
    quickCheck (withMaxSuccess 20000 prop_PredicateAndCommutativity) 

-- Define our predicate and properties
prop_PredicateAndCommutativity :: Int -> Int -> Int -> Property
prop_PredicateAndCommutativity x y z = (x + y == z) ==> (x + y == y + x && x + y == z)

-- random property for testing
prop_addition :: Int -> Int -> Int -> Bool
prop_addition x y z = (x + y) + z == x + (y + z)

