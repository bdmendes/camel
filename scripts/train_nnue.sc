//> using scala "3.3.7"
//> using jvm 21
//> using javaOpt "-Xms2G"
//> using javaOpt "-Xmx8G"
//> using dep com.github.mrdimosthenis::synapses:8.0.0
//> using dep "io.github.lunalobos:chessapi4j:1.2.11"
//> using dep "io.circe::circe-core:0.14.15"
//> using dep "io.circe::circe-generic:0.14.15"
//> using dep "io.circe::circe-parser:0.14.15"

import chessapi4j.Piece.*
import chessapi4j.Position
import io.circe.*
import io.circe.parser.*
import java.io.PrintWriter
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import scala.collection.mutable
import scala.io.Source
import scala.util.*
import synapses.lib.*

// As expected by our `NeuralNetwork` module.
val Pieces = List(WP, WN, WB, WR, WQ, WK, BP, BN, BB, BR, BQ, BK)
val Scale = 2000.0

// Hyperparameters.
val LearningRate = 0.005
val Epochs = 10
val LearningRateDecayFactor = 0.8
val Observations = 800_000

val EpdPathFromRoot = "./assets/books/quiet-evaluated-filtered-camelv1.epd"
val SamplePositions = List(
  "4rrk1/p1ppqpb1/B4np1/3nN3/8/8/PPPB1P1P/R3KQ1R w KQ - 0 1" -> -40,
  "1k6/8/1P6/3R3p/6p1/6r1/8/4K3 b - - 0 1" -> -80,
  "b7/2p2ppk/1p2p2p/p3P2P/P3pK2/bP2P3/2P1BPP1/3R4 w - - 0 1" -> 250,
  "8/1p6/2b5/2b5/2P5/8/2K2k2/8 b - - 0 1" -> -700,
  "8/p7/1p2k3/2p5/b1P1BP1B/3K4/1b6/8 w - - 0 1" -> 0,
  "r2q1rk1/p1p1bpp1/4p2p/2np4/5P2/4PR1P/PP2B1P1/RN2Q1K1 b - - 0 1" -> -100,
  "rnbqkbnr/ppp2p1p/6p1/3pPp2/3P4/5N2/PPP1B1PP/RN1QK2R w KQkq - 0 1" -> -400,
  "rnb2rk1/2pnbppp/1p2p3/pP6/P3P3/2NB1N2/5PPP/R1BQK2R b KQ - 0 1" -> 1000,
  "2bq1rk1/4pp1p/6p1/p2p4/Pp6/1P2PNP1/1Q3PBP/R5K1 w - - 0 1" -> 300,
  "3r4/1pk3p1/p4pB1/4p1p1/bPPr4/3P3P/5PP1/2R3K1 w - - 0 1" -> -550,
  "1rbqk1nr/p1pp1p1p/2p3p1/8/P2bP3/6P1/2P2PBP/RNB1K2R w KQk - 0 1" -> -1100,
  "2r3k1/pq3ppp/8/1pp4Q/8/3P2P1/Pb2PP1P/1R1R2K1 b - - 0 1" -> 250,
  "rnbb1rk1/pp4pp/4pn2/2p2p2/2P4N/2N3P1/PP2PPBP/R1B2RK1 b - - 0 1" -> 50,
  "4r3/6b1/1p3kpp/pN1R4/P6P/2P3P1/1P3K2/8 w - - 0 1" -> 150,
  "2b2r2/6kp/3q2p1/1p1p1r2/p1pPp3/P3P2P/1P3PP1/QR3RK1 w - - 0 1" -> -550,
  "r1bq1rk1/pp1nppbp/5np1/1B1p4/3P1B2/4PN1P/PPPQ1PP1/R4RK1 b - - 0 1" -> -250,
  "rn2qrk1/pb2bppp/1pp1p3/8/2BP4/4PNN1/PP3PPP/2RQ2K1 b - - 0 1" -> -500,
  "3r1rk1/1p2bpp1/7p/1N1p3P/P2R4/6P1/1P2PP2/3R2K1 b - - 0 1" -> 200,
  "8/4pp2/4k1p1/1R5p/4KP1P/4P1P1/8/r7 b - - 0 1" -> 0
)

def toInput(fen: String): List[Double] =
  val position = new Position(fen)
  val input = mutable.ArraySeq.fill(768)(0.0)
  Pieces
    .zipWithIndex
    .foreach: (piece, idx) =>
      val bb = position.getBitboard(piece)
      while bb.getValue() != 0 do
        val sq = bb.trailingZeros()
        bb.popLastBit()
        input.update(idx * 64 + sq, 1.0)
  input.toList
end toInput

def toInputExpected(epdLine: String): (List[Double], Double) =
  val parts = epdLine.split(" ")
  val fen = parts.take(6).mkString(" ")
  val eval = parts.last.drop(1).dropRight(2).toDouble / Scale
  toInput(fen) -> eval.max(-1.0).min(1.0)

def serialize(net: Net): Unit =
  case class Layer(weights: List[Double]) derives Decoder:
    lazy val actualWeights = weights.drop(1)
    val bias = weights.head

  val json = parse(net.json()).toOption.get
  val layers = json.as[List[List[Layer]]].toOption.get
  val accWeights = layers.head.map(_.actualWeights).flatten
  val accBiases = layers.head.map(_.bias).toList
  val outWeights = layers.last.head.actualWeights
  val outBias = layers.last.head.bias

  val nnueJson = Json.obj(
    "acc_weights" -> Json.fromValues(accWeights.map(w => Json.fromDoubleOrNull(w))),
    "acc_biases" -> Json.fromValues(accBiases.map(b => Json.fromDoubleOrNull(b))),
    "out_weights" -> Json.fromValues(outWeights.map(w => Json.fromDoubleOrNull(w))),
    "out_bias" -> Json.fromDoubleOrNull(outBias)
  )

  val fileName =
    val now = LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss"))
    s"./assets/models/quiet-labeled-$now.nnue"

  Using.resource(new PrintWriter(fileName)): writer =>
    writer.write(nnueJson.noSpaces)
end serialize

def mse(net: Net): Double =
  val err =
    SamplePositions.foldLeft(0.0):
      case (acc, (fen, expectedEval)) =>
        val output = net.predict(toInput(fen)).head * Scale
        val error = (output - expectedEval) * (output - expectedEval)
        acc + error
  err / SamplePositions.length

(1 to Epochs).foldLeft(Net(List(768, 128, 1), _ => Fun.leakyReLU, _ => Random.nextDouble() * 2 - 1)):
  case (nn, epoch) =>
    val learningRate = LearningRate * math.pow(LearningRateDecayFactor, epoch - 1)
    println(s"Starting epoch $epoch. Learning rate = $learningRate.")
    val (next, _) =
      Using.resource(Source.fromFile(EpdPathFromRoot)): src =>
        src
          .getLines()
          .take(Observations)
          .map(toInputExpected)
          .foldLeft(nn -> 1):
            case (acc -> counter, xs -> ys) =>
              val handle =
                if counter % 10_000 == 0 then
                  println(s"[Epoch $epoch/$Epochs] Processed $counter elements. Error: ${mse(acc).toInt}")
                  // The persistent NN seems to leak some memory, so occasionally recreate it.
                  Net(acc.json())
                else
                  acc
              handle.parFit(learningRate, xs, List(ys)) -> (counter + 1)
    end val
    serialize(next)
    next
