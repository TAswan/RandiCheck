data Range = Range Int Int
  deriving (Show, Eq)

validRange :: Range -> Bool
validRange (Range start end) = start <= end
