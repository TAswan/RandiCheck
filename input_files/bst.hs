data Tree = Leaf
          | Node Int Tree Tree
    deriving (Show, Eq)

isBST :: Tree -> Bool
isBST Leaf = True
isBST (Node v left right) = (f v left) && (g v right) && (isBST left) && (isBST right)
   where
     f c Leaf = True
     f c (Node h l r) = (h <= c) && (f c l) && (f c r)
     g c Leaf = True
     g c (Node h l r) = (h >= c) && (g c l) && (g c r)

     