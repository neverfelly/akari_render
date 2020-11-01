// MIT License
//
// Copyright (c) 2020 椎名深雪
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#ifndef AKARIRENDER_IMAGE_HPP
#define AKARIRENDER_IMAGE_HPP

#include <list>
#include <vector>
#include <akari/core/akari.h>
#include <akari/core/math.h>
#include <akari/core/color.h>
#include <akari/core/memory.h>

namespace akari {

    template <class T>
    class TImage {
        using Float = float;

        astd::pmr::vector<T> _texels;
        ivec2 _resolution;

      public:
        TImage(const ivec2 &dim = ivec2(1), Allocator<> allocator = Allocator<>())
            : _texels(dim[0] * dim[1], allocator), _resolution(dim) {}

        const T &operator()(int x, int y) const {
            x = std::clamp(x, 0, _resolution[0] - 1);
            y = std::clamp(y, 0, _resolution[1] - 1);
            return _texels[x + y * _resolution[0]];
        }

        T &operator()(int x, int y) {
            x = std::clamp(x, 0, _resolution[0] - 1);
            y = std::clamp(y, 0, _resolution[1] - 1);
            return _texels[x + y * _resolution[0]];
        }

        const T &operator()(float x, float y) const { return (*this)(vec2(x, y)); }

        T &operator()(float x, float y) { return (*this)(vec2(x, y)); }

        const T &operator()(const ivec2 &p) const { return (*this)(p.x, p.y); }

        T &operator()(const ivec2 &p) { return (*this)(p.x, p.y); }

        const T &operator()(const vec2 &p) const { return (*this)(ivec2(p * vec2(_resolution))); }

        T &operator()(const vec2 &p) { return (*this)(ivec2(p * vec2(_resolution))); }

        [[nodiscard]] const astd::pmr::vector<T> &texels() const { return _texels; }

        void resize(const ivec2 &size) {
            _resolution = size;
            _texels.resize(_resolution[0] * _resolution[1]);
        }

        [[nodiscard]] ivec2 resolution() const { return _resolution; }
        T *data() { return _texels.data(); }

        [[nodiscard]] const T *data() const { return _texels.data(); }

        struct View {
            const T &operator()(int x, int y) const {
                x = std::clamp(x, 0, _resolution[0] - 1);
                y = std::clamp(y, 0, _resolution[1] - 1);
                return _texels[x + y * _resolution[0]];
            }

            T &operator()(int x, int y) {
                x = std::clamp(x, 0, _resolution[0] - 1);
                y = std::clamp(y, 0, _resolution[1] - 1);
                return _texels[x + y * _resolution[0]];
            }

            const T &operator()(float x, float y) const { return (*this)(vec2(x, y)); }

            const T &operator()(const ivec2 &p) const { return (*this)(p.x, p.y); }

            const T &operator()(const vec2 &p) const { return (*this)(ivec2(p * vec2(_resolution))); }

            [[nodiscard]] ivec2 resolution() const { return _resolution; }
            [[nodiscard]] const T *data() const { return _texels; }
            const T *_texels = nullptr;
            ivec2 _resolution = ivec2(0);
        };
        View view() const { return {data(), resolution()}; }
    };

    class RGBImage : public TImage<Color<float, 3>> {
      public:
        using TImage<Color<float, 3>>::TImage;
    };

    struct alignas(16) RGBA {
        RGB rgb;
        float alpha;
        RGBA() = default;
        RGBA(vec3 rgb, float alpha) : rgb(rgb), alpha(alpha) {}
    };
    class RGBAImage : public TImage<RGBA> {
      public:
        using TImage<RGBA>::TImage;
    };

    class AKR_EXPORT PostProcessor {
      public:
        virtual void process(const RGBAImage &in, RGBAImage &out) const = 0;
    };
    class IdentityProcessor : public PostProcessor {
      public:
        void process(const RGBAImage &in, RGBAImage &out) const override { out = in; }
    };
    class AKR_EXPORT GammaCorrection : public PostProcessor {

      public:
        explicit GammaCorrection() {}
        void process(const RGBAImage &in, RGBAImage &out) const override;
    };
    class AKR_EXPORT Convolution : public PostProcessor {
        TImage<Float> kernel;
        ivec2 stride;

      public:
        explicit Convolution(TImage<Float> kernel, ivec2 stride) : kernel(kernel), stride(stride) {}
        void process(const RGBAImage &in, RGBAImage &out) const override;
    };
    class PostProcessingPipeline : public PostProcessor {
        std::list<std::shared_ptr<PostProcessor>> pipeline;

      public:
        void Add(const std::shared_ptr<PostProcessor> &p) { pipeline.emplace_back(p); }
        void process(const RGBAImage &in, RGBAImage &out) const override {
            RGBAImage tmp;
            for (auto it = pipeline.begin(); it != pipeline.end(); it++) {
                if (it == pipeline.begin()) {
                    tmp = in;
                } else {
                    tmp = out;
                }
                (*it)->process(tmp, out);
            }
        }
    };

    class AKR_EXPORT ImageWriter {
      public:
        virtual bool write(const RGBAImage &image, const fs::path &, const PostProcessor &postProcessor) = 0;
        virtual ~ImageWriter() = default;
    };

    class AKR_EXPORT ImageReader {
      public:
        virtual std::shared_ptr<RGBAImage> read(const fs::path &) = 0;
        virtual ~ImageReader() = default;
    };

    AKR_EXPORT std::shared_ptr<ImageWriter> default_image_writer();
    AKR_EXPORT std::shared_ptr<ImageReader> default_image_reader();

} // namespace akari

#endif // AKARIRENDER_IMAGE_HPP
