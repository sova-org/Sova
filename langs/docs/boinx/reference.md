## The pattern-friendly '_' function

The '_' function is a special function which can only be called pre-flattening (as it cannot be parsed without the underscore), which is more or less a syntaxic sugar for `. + _ex(arg)` (as Boinx use a timed arithmetic).

It is mainly used to stack patterns and provide a "tidal-like" way of writing things. For instance:
```
s: 'saw'
| _(note: [C5 E5 G5 C6])
| _(lpf: [100 1500])
# . + pan: rand(0.2, 0.8) // Applies for each event generated
```
The code above will result in 4 event spread evenly:
```
[
<s: 'saw' note: C5 lpf: 100 pan: rand(0.2, 0.8)>
<s: 'saw' note: E5 lpf: 100 pan: rand(0.2, 0.8)>
<s: 'saw' note: G5 lpf: 1500 pan: rand(0.2, 0.8)>
<s: 'saw' note: C6 lpf: 1500 pan: rand(0.2, 0.8)>
]
```
