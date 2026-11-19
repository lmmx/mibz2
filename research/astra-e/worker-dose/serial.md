# What an unknown shared serial cost can explain

Let a and b be homogeneous eager and edge wall times at the same W, with eager fraction p strictly between zero and one. Suppose both have the same additive serial time s, and independent worker service rates otherwise. Then the predicted mixture time is

    T(s) = s + (a−s)(b−s) / [p(b−s) + (1−p)(a−s)].

With d = pb + (1−p)a, differentiation gives T′(s) = (d−a)(d−b)/(d−s)² ≤ 0. As s goes from zero to min(a,b), the mixed time moves continuously from the weighted harmonic mean to min(a,b). If the two homogeneous times agree, the mixed time is constant. With p=1 the result is a independently of b.

A perfectly overlapped common scanner has the same envelope. If both homogeneous worker stages exceed scanner time, their mixture is the harmonic mean. If the faster arm is capped at the scanner, its underlying worker stage is at most its observed time. Blending this stage with the slower arm gives a worker time at most the harmonic mean of the measured times. Taking the maximum with the scanner puts the resulting time between the faster measured arm and that harmonic mean.

This distinguishes two claims. A mixture below the zero-serial harmonic mean can be consistent with ordinary independent service and a shared serial cost. A mix that repeatedly beats both homogeneous arms would require an effect outside these models, such as altered resource interference, job allocation or delivery. Wall measurements have sampling errors and the faster of two noisy references has selection bias. The bound is a property of ideal matched service times. A confidence interval for measured mixed/faster is a descriptive check on the raw observations.

At W2 the local paired mixed/faster ratio was 1.2216 [0.7730,1.9306], and mixed/harmonic was 1.0374 [0.6919,1.5554]. At W4 the corresponding ratios were 1.1298 [0.9624,1.3263] and 0.9118 [0.6851,1.2135]. Eight W2 rounds and four W4 rounds put the measured mix below the faster homogeneous observation. Neither grid resolved a joint-advantage claim. [All local cells](panel/envelope.csv) · [Reconstruction](envelope.py) · [Finite model checks](envelope-check.txt)
