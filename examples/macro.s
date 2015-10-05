(macro foo x (let y 4 (+ y x)))
(foo 42)

(let y 42 ((macro bar x (+ y x)) (let y 0 (bar 10))))
