data Tree = Leaf
          | Node Int Tree Tree
    deriving (Show, Eq)

isBST :: Tree -> Bool
isBST Leaf = True
isBST (Node v left right) = f (<=v) left && f (>=v) right && isBST left && isBST right
   where
     f _ Leaf = True
     f c (Node h l r) = c h && f c l && f c r

     