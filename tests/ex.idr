

data Test =  IntC Int | BoolC Bool

foo : Test -> Bool
foo (BoolC b) = b
foo (IntC i) = i < 5

