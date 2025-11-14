data Test = IntC Int | BoolC Bool

foo :: Test -> Bool
foo (BoolC b) = b
foo (IntC i) = x i
  where 
        x n = n > 1

