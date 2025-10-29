

data Test =  IntC Int | BoolC Bool | BoolD Bool Boo

data Boo = Bo Bool

foo :: Test -> Bool
foo (BoolC b) = b
foo (IntC i) = i +1 > 5
foo (BoolD a b) = a == b

