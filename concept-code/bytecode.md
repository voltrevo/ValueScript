# Bytecode Notes

The bytecode is also a serialization format, able to represent any value that
can ever exist within ValueScript (and in ValueScript, everything is a value).

Serialization is 'batteries included' - meaning that it includes everything
contained by that entity.
- Notably, when it comes to functions, this means that the format must provide
everything needed to execute that function in another context without any
further communication with the original context. This contrasts with
JavaScript's Function.prototype.toString which only returns the source code,
which usually includes unresolved references to other entities.

Interpreting bytecode always starts with parsing a value, and the value parsed
at the beginning is the overall value being represented.

## Values

These are the values that can currently be represented along with the leading
byte that indicates them:

```
01 void
02 undefined
03 null
04 false
05 true
06 signed byte (as a number)
07 number
08 string
09 array
0a object
0b function
0c instance
```

The following bytes can also be encountered when decoding a value.

```
00 end

0d pointer
0e register
0f external
```

These aren't valid in all contexts. For example, when decoding an array, we keep
decoding values until encountering `end`, but it would not be valid to have
`end` immediately when decoding a top-level value.

## Instructions

```
00 end
01 mov

02 op++
03 op--

04 op+
05 op-
06 op*
07 op/
08 op%
09 op**
0a op==
0b op!=
0c op===
0d op!==

0e op&&
0f op||
10 op!

11 op<
12 op<=
13 op>
14 op>=

15 op??
16 op?.

17 op&
18 op|
19 op~
1a op^
1b op<<
1c op>>
1d op>>>

1e typeof
1f instanceof
20 in

21 call
22 apply
23 bind
24 sub
25 submov
26 subcall

27 jmp
28 jmpif
```
