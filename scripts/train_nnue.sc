#!/usr/bin/env -S scala-cli shebang
//> using scala "3.3.7"
//> using jvm 21
//> using javaOpt "-Xms2G"
//> using javaOpt "-Xmx8G"
//> using repository "sonatype-s01:snapshots"
//> using repository "sonatype:snapshots"
//> using dep "io.github.lunalobos:chessapi4j:1.2.11"
//> using dep "dev.storch::core:0.0-2dfa388-SNAPSHOT"
//> using dep "org.bytedeco:pytorch-platform:2.7.1-1.5.12"
//> using dep "io.circe::circe-core:0.14.15"

import chessapi4j.Piece.*
import chessapi4j.Position
import io.circe.Json
import java.io.PrintWriter
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import scala.collection.mutable
import scala.collection.mutable.ArrayBuffer
import scala.io.Source
import scala.util.Using
import torch.*
import torch.nn.functional as F

def toInput(fen: String): Tensor[Float64] =
  val position = new Position(fen)
  val input = mutable.ArraySeq.fill(768)(0.0d)
  Seq(WP, WN, WB, WR, WQ, WK, BP, BN, BB, BR, BQ, BK).zipWithIndex
    .foreach: (piece, idx) =>
      val bb = position.getBitboard(piece)
      while bb.getValue() != 0 do
        val sq = bb.trailingZeros()
        bb.popLastBit()
        input.update(idx * 64 + sq, 1.0d)
  torch.Tensor(input.toSeq)
end toInput

def toInputExpected(epdLine: String): (Tensor[Float64], Tensor[Float64]) =
  val parts = epdLine.split(" ")
  val fen = parts.take(6).mkString(" ")
  val eval = parts.last.drop(1).dropRight(2).toDouble / 1200.0d
  toInput(fen) -> torch.Tensor(Seq(eval.max(-1.0d).min(1.0d))).reshape(1, 1)

class NNUE extends nn.Module:
  private val layer1 = register(nn.Linear[Float64](768, 64))
  private val layer2 = register(nn.Linear[Float64](64, 1))

  def optimizer(learningRate: Double) = optim.Adam(
    params = parameters,
    lr = learningRate,
    betas = (0.9, 0.999),
    eps = 1e-8,
    weightDecay = 0.0,
    amsgrad = false
  )

  def apply(input: Tensor[Float64]): Tensor[Float64] =
    val acc = F.relu(layer1(input))
    layer2(acc)

  def json: Json =
    val accWeights = layer1.weight.flatten.toArray.map(_.toDouble).toList
    val accBiases = layer1.bias.toArray.map(_.toDouble).toList
    val outWeights = layer2.weight.flatten.toArray.map(_.toDouble).toList
    val outBias = layer2.bias.toArray.headOption.map(_.toDouble).getOrElse(0.0d)

    Json.obj(
      "acc_weights" -> Json.arr(accWeights.map(Json.fromDoubleOrNull)*),
      "acc_biases" -> Json.arr(accBiases.map(Json.fromDoubleOrNull)*),
      "out_weights" -> Json.arr(outWeights.map(Json.fromDoubleOrNull)*),
      "out_bias" -> Json.fromDoubleOrNull(outBias)
    )
  end json
end NNUE

object NNUE:
  val LearningRate = 0.005
  val LearningRateDecay = 0.98
  val Epochs = 150
  val BatchSize = 512

  def mseLoss(pred: Tensor[Float64], target: Tensor[Float64]): Tensor[Float64] =
    val diff = pred - target
    val sq = diff * diff
    sq.mean

  def epdBatches(): Iterator[(Tensor[Float64], Tensor[Float64])] =
    val src =
      Source.fromFile("./assets/books/quiet-evaluated-filtered-camelv1.epd")

    new Iterator[(Tensor[Float64], Tensor[Float64])]:
      private val it = src.getLines()
      private val xsBuf = ArrayBuffer.empty[Tensor[Float64]]
      private val ysBuf = ArrayBuffer.empty[Tensor[Float64]]
      private var nextBatch: Option[(Tensor[Float64], Tensor[Float64])] =
        fetch()

      private def fetch(): Option[(Tensor[Float64], Tensor[Float64])] =
        xsBuf.clear()
        ysBuf.clear()

        while it.hasNext && xsBuf.size < BatchSize do
          val line = it.next()
          val (x, y) = toInputExpected(line)
          xsBuf += x
          ysBuf += y

        if xsBuf.isEmpty then
          src.close()
          None
        else
          val xb = torch.stack(xsBuf.toSeq).reshape(xsBuf.size, 768)
          val yb = torch.stack(ysBuf.toSeq).reshape(ysBuf.size, 1)
          Some((xb, yb))
      end fetch

      override def hasNext: Boolean = nextBatch.nonEmpty
      override def next(): (Tensor[Float64], Tensor[Float64]) =
        val out = nextBatch.get
        nextBatch = fetch()
        out
    end new
  end epdBatches
end NNUE

val net = NNUE()

for epoch <- 1 to NNUE.Epochs do
  var batchIdx = 0
  var runningLoss = 0.0
  val lr = NNUE.LearningRate * math.pow(NNUE.LearningRateDecay, epoch - 1)
  val optimizer = net.optimizer(lr)

  for (xb, yb) <- NNUE.epdBatches() do
    batchIdx += 1

    optimizer.zeroGrad()
    val prediction = net(xb)
    val loss = NNUE.mseLoss(prediction, yb)
    loss.backward()
    optimizer.step()

    runningLoss += loss.item.toDouble

    if batchIdx % 200 == 0 then
      val avg = runningLoss / 200.0d
      runningLoss = 0.0d
      println(f"[epoch $epoch%3d, batch $batchIdx%4d, lr $lr%2f] loss=$avg%.6f")
  end for

  if epoch % 20 == 0 || epoch == NNUE.Epochs then
    val serialized = net.json.noSpaces
    val now =
      LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss"))
    Using.resource(new PrintWriter(s"./assets/dump/$now.nnue")): pw =>
      pw.write(serialized)
end for
