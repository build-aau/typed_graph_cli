# Primitive types

## bool
Represents either a true or false value

### JSON Representation
`true` or `false`

## u8, u16, u32, u64, usize
Represents whole positive numbers of various sizes. The size is given in terms of number of bits and sets the upper bound for which numbers can be stored

usize is a special case that uses a platform specific size. This is either similar to u32 or u64

### JSON Representation
Numbers like `1`, `719120` and `4010`

## i8, i16, i32, i64, isize 
Represents whole numbers of various sizes. The size is given in terms of number of bits and sets the upper bound for which numbers can be stored.  
Since one bit is used for the sign these are often smaller than their unsigned counterpart.  

isize is a special case that uses a platform specific size. This is either similar to i32 or i64

### JSON Representation
Numbers like `-1`, `719120` and `-4010`

## f32, f64
Represents decimal numbers. The size of a float describe how precisely it represents a given value.

### JSON Representation
Decimals like `-1.0`, `1.0`, `719120.1235` and `-4010.7493`

## String
Represents a piece of utf-8 encoded text

### JSON Representation
`"/* String value */"`

# Built-in types

## Option\<T\>
Represents either the presence or absence of a value.

The type of T depends on the context in which it is used

### JSON Representation
Either `/* T body */` or `null`

## List\<T\>
Represents 0 or more of a value.

The type of T depends on the context in which it is used

## Set\<T\>
Represents 0 or more of unique values.

The type of T depends on the context in which it is used

### JSON Representation
Either `[]` or 
```
[
    /* T body */, 
    /* T body */, 
    ... 
]
```

## Map\<K, V\>
Represents a mapping from a key type K to a value type V

The type of K and V depends on the context in which it is used

### JSON Representation
Either `{}` or 
```
{
    /* K body */: /* V body*/,
    /* K body */: /* V body*/,
    ...
]
```
