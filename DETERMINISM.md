# LLM Inferencing Determinism

AVSs are run by an operator set consisting of many nodes running essentially an AI Oracle, and the LLM Inference examples here require deterministic output to work.

Achieving determinism with LLM inferencing is actually fairly straightforward, but there are some caveats, namely:

- You need to hard code the `seed`
- AVS operators MUST be all using the same GPU models (as even with a hardcoded seed, there will be variance based on different GPU models).

We _strongly recommend_ using open source models for deterministic inferencing and adding some hardware requirements for operators running your AVS (this is the most decentralized, robust, and censorship resistant). In a production AVS environment, you would need to ship a docker image that bundles WAVS and a local LLM instance _together_ into a new docker image. More information on support for WAVS sidecars will be forthcoming in a future release. For deterministic output, every AVS operator MUST use the same GPU.

While this example also includes support for using OpenAI models (unfortunately Anthropic does not support setting the `seed` with their API yet), there may not be strict guarantees on which GPU will be running inference on. Therefore, we do not recommend using such proprietary models for AVS use cases (as there is a small chance of non-determinism).

However, it is possible to achieve determinism even with non-deterministic models (or in cases where you can't be certain which GPU an inference request is being run on) by _reducing the output space_. For example, all LLMs will follow instructions such as "only output YES or NO". By reducing the possibility space, and setting `temperature` to zero to get the most predictable output, you will be able to achieve deterministic results.

In the DAO Agent example, this is achieved through tool use. All other text in the response is ignored and only the output from tool calls is used.

More testing and experimentation is needed. We are very excited for the potential of Deterministic Inferencing, and hope that others take interest as well. For more information, check out our original [blog post](https://www.layer.xyz/news-and-insights/deterministic-ai) on the subject.
