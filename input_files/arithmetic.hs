data Equation = Eqn Int Int Int
  deriving (Show, Eq)

validEquation :: Equation -> Bool
validEquation (Eqn a b c) = ((a * a ) + (b * b)) == (c * c)