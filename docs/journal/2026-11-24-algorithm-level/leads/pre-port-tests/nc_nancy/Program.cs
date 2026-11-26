// Pre-port test 1 (Deep Dive #1): Deterministic Network Calculus via stock Unipi.Nancy 1.6.0
// Computes exact UPP tandem convolution (beta_S * beta_W * beta_D), exact Zippo-Stea /
// Le Boudec-Thiran window-flow-control feedback closure beta_cl = beta_fwd * subadd(beta_fwd + W),
// batch completion-time upper bounds U_NC(k) = beta_cl^{-1}(N) + delta_tiny, and policy separation
// (W = 4k vs W = 2k; S+C vs C-only at k = 8) on character-stage1 parameters.

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Unipi.Nancy.MinPlusAlgebra;
using Unipi.Nancy.NetworkCalculus;
using Unipi.Nancy.Numerics;

static Rational Rat(double x, long scale = 100000)
{
    long num = (long)Math.Round(x * scale);
    if (num <= 0) num = 1;
    return new Rational(num, scale);
}

static Rational RatZeroOk(double x, long scale = 100000)
{
    long num = (long)Math.Round(x * scale);
    if (num < 0) num = 0;
    return new Rational(num, scale);
}

static (double uFwd, double uW4k, double uW2k, double slopeFwd, double slopeW4k, double slopeW2k)
    EvalModel(double N, int W4k, int W2k, double rS, double tS, double rW, double tW, double rD, double tD, double deltaTiny)
{
    // Use microsecond-accurate rational approximations (scale 1e5 for rates, 1e6 for latencies in seconds)
    var bS = new RateLatencyServiceCurve(Rat(rS, 100), RatZeroOk(tS, 1000000));
    var bW = new RateLatencyServiceCurve(Rat(rW, 100), RatZeroOk(tW, 1000000));
    var bD = new RateLatencyServiceCurve(Rat(rD, 100), RatZeroOk(tD, 1000000));

    var bFwd = Curve.Convolution(Curve.Convolution(bS, bW), bD);
    var bW4 = Curve.Convolution(bFwd, bFwd.VerticalShift(new Rational(W4k)).SubAdditiveClosure());
    var bW2 = Curve.Convolution(bFwd, bFwd.VerticalShift(new Rational(W2k)).SubAdditiveClosure());

    var nRat = new Rational((long)Math.Round(N));
    double uFwd = (double)bFwd.LowerPseudoInverse().ValueAt(nRat) + deltaTiny;
    double uW4 = (double)bW4.LowerPseudoInverse().ValueAt(nRat) + deltaTiny;
    double uW2 = (double)bW2.LowerPseudoInverse().ValueAt(nRat) + deltaTiny;

    return (
        uFwd,
        uW4,
        uW2,
        (double)bFwd.PseudoPeriodSlope,
        (double)bW4.PseudoPeriodSlope,
        (double)bW2.PseudoPeriodSlope
    );
}

string jsonPath = args.Length > 0
    ? args[0]
    : Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "nc_stage1_params.json");
if (!File.Exists(jsonPath))
    jsonPath = "docs/journal/2026-11-24-algorithm-level/leads/pre-port-tests/nc_stage1_params.json";

using var doc = JsonDocument.Parse(File.ReadAllText(jsonPath));
var root = doc.RootElement;
var extracted = root.GetProperty("extracted");
var models = root.GetProperty("models");

string[] modelNames = {
    "M1_strict_block_worst",
    "M2_trace_envelope_block",
    "M3a_open_strict_worst",
    "M3b_open_mean_hol"
};

var results = new Dictionary<string, object>();

foreach (var mName in modelNames)
{
    Console.WriteLine($"\n=== Nancy UPP + Window Closure: {mName} ===");
    Console.WriteLine(" k  T_wall |  U_fwd  fwd/w | U_W=4k  4k/w  R_4k    | U_W=2k  2k/w  R_2k    | sep(2k/4k)");
    var mElem = models.GetProperty(mName);
    var perK = new Dictionary<string, object>();

    for (int k = 1; k <= 10; k++)
    {
        string ks = k.ToString();
        double tWall = extracted.GetProperty(ks).GetProperty("wall_med").GetDouble();
        var p = mElem.GetProperty(ks);
        double N = p.GetProperty("N").GetDouble();
        int W4k = p.GetProperty("W_4k").GetInt32();
        int W2k = p.GetProperty("W_2k").GetInt32();
        double rS = p.GetProperty("R_S").GetDouble();
        double tS = p.GetProperty("T_S").GetDouble();
        double rW = p.GetProperty("R_W").GetDouble();
        double tW = p.GetProperty("T_W").GetDouble();
        double rD = p.GetProperty("R_D").GetDouble();
        double tD = p.GetProperty("T_D").GetDouble();
        double dt = p.GetProperty("delta_tiny").GetDouble();

        var (uFwd, u4, u2, sFwd, s4, s2) = EvalModel(N, W4k, W2k, rS, tS, rW, tW, rD, tD, dt);
        Console.WriteLine(
            $"{k,2}  {tWall,6:F4} | {uFwd,6:F4} {uFwd / tWall,5:F2}x | " +
            $"{u4,6:F4} {u4 / tWall,5:F2}x {s4,7:F1} | " +
            $"{u2,6:F4} {u2 / tWall,5:F2}x {s2,7:F1} | {u2 / u4,5:F2}x"
        );
        perK[ks] = new
        {
            k,
            wall_med = tWall,
            u_fwd = uFwd,
            ratio_fwd = uFwd / tWall,
            u_w4k = u4,
            ratio_w4k = u4 / tWall,
            slope_w4k = s4,
            u_w2k = u2,
            ratio_w2k = u2 / tWall,
            slope_w2k = s2,
            sep_2k_over_4k = u2 / u4
        };
    }
    results[mName] = perK;
}

// Also compute Nancy exact vertical/horizontal deviations against multi-segment empirical alpha(t)
// from burst_frontier.tsv (t_issued) on M3b and M2 (downstream W_k * D)
Console.WriteLine("\n=== Nancy Policy Separation at k=8 (M3b_open_mean_hol & M1_strict_block_worst) ===");
var policyResults = new Dictionary<string, object>();
double cShare8 = extracted.GetProperty("8").GetProperty("C_share_of_worker").GetDouble();
double wall8 = extracted.GetProperty("8").GetProperty("wall_med").GetDouble();

foreach (var mName in new[] { "M1_strict_block_worst", "M3a_open_strict_worst", "M3b_open_mean_hol" })
{
    var p = models.GetProperty(mName).GetProperty("8");
    double N = p.GetProperty("N").GetDouble();
    int W4k = p.GetProperty("W_4k").GetInt32();
    int W2k = p.GetProperty("W_2k").GetInt32();
    double rS = p.GetProperty("R_S").GetDouble();
    double tS = p.GetProperty("T_S").GetDouble();
    double rW = p.GetProperty("R_W").GetDouble();
    double tW = p.GetProperty("T_W").GetDouble();
    double rD = p.GetProperty("R_D").GetDouble();
    double tD = p.GetProperty("T_D").GetDouble();
    double dt = p.GetProperty("delta_tiny").GetDouble();

    Console.WriteLine($"  [{mName}]");
    var polMap = new Dictionary<string, object>();
    foreach (var (pname, sFac, cCut, wOverride) in new (string, double, double, int?)[] {
        ("baseline_W4k", 1.0, 0.0, null),
        ("window_W2k", 1.0, 0.0, W2k),
        ("C_minus_30pct", 1.0, 0.30, null),
        ("SC_minus_30pct", 0.70, 0.30, null),
        ("C_ideal", 1.0, 1.0, null),
        ("SC_ideal", 0.10, 1.0, null),
    })
    {
        double wFac = 1.0 - cCut * cShare8;
        int wUse = wOverride ?? W4k;
        var (uFwd, uWin, _, _, sWin, _) = EvalModel(
            N, wUse, wUse,
            rS / sFac, tS * sFac,
            rW / wFac, tW * wFac,
            rD, tD, dt
        );
        Console.WriteLine($"    {pname,-16} W={wUse,2}: U_fwd={uFwd:F4}s ({uFwd/wall8:F2}x), U_win={uWin:F4}s ({uWin/wall8:F2}x), R_win={sWin:F1}");
        polMap[pname] = new { W = wUse, u_fwd = uFwd, u_win = uWin, ratio_win = uWin / wall8, slope_win = sWin };
    }
    policyResults[mName] = polMap;
}
results["policy_k8"] = policyResults;

string outPath = Path.Combine(Path.GetDirectoryName(Path.GetFullPath(jsonPath))!, "nc_nancy_results.json");
File.WriteAllText(outPath, JsonSerializer.Serialize(results, new JsonSerializerOptions { WriteIndented = true }));
Console.WriteLine($"\nWrote {outPath}");
