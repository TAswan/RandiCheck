data B = Bezout Int Int
    deriving (Show, Eq)

validBezout :: B -> Bool
validBezout (Bezout a b) = ((a * 15) + (b * 23)) == 1