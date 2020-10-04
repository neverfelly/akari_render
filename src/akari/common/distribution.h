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

#pragma once
#include <akari/common/math.h>
#include <akari/common/buffer.h>
namespace akari {
    /*
     * Return the largest index i such that
     * pred(i) is true
     * If no such index i, last is returned
     * */
    template <typename Pred>
    AKR_XPU int upper_bound(int first, int last, Pred pred) {
        int lo = first;
        int hi = last;
        while (lo < hi) {
            int mid = (lo + hi) / 2;
            if (pred(mid)) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        return std::clamp<int>(hi - 1, 0, (last - first) - 2);
    }
    AKR_VARIANT
    struct Distribution1D {
        friend struct Distribution2D;
        Distribution1D(MemoryResource *resource, const Float *f, size_t n) : func(resource), cdf(resource) {
            func.copy(f, n);
            cdf.resize(n + 1);
            cdf[0] = 0;
            for (size_t i = 0; i < n; i++) {
                cdf[i + 1] = cdf[i] + func[i] / n;
            }
            funcInt = cdf[n];
            if (funcInt == 0) {
                for (uint32_t i = 1; i < n + 1; ++i)
                    cdf[i] = Float(i) / Float(n);
            } else {
                for (uint32_t i = 1; i < n + 1; ++i)
                    cdf[i] /= funcInt;
            }
        }
        // y = F^{-1}(u)
        // P(Y <= y) = P(F^{-1}(U) <= u) = P(U <= F(u)) = F(u)
        // Assume: 0 <= i < n
        [[nodiscard]] AKR_XPU Float pdf_discrete(int i) const { return func[i] / (funcInt * count()); }
        [[nodiscard]] AKR_XPU Float pdf_continuous(Float x) const {
            uint32_t offset = std::clamp<uint32_t>(static_cast<uint32_t>(x * count()), 0, count() - 1);
            return func[offset] / funcInt;
        }
        AKR_XPU int sample_discrete(Float u, Float *pdf = nullptr) {
            uint32_t i = upper_bound(0, cdf.size(), [=](int idx) { return cdf[idx] <= u; });
            if (pdf) {
                *pdf = pdf_discrete(i);
            }
            return i;
        }

        AKR_XPU Float sample_continuous(Float u, Float *pdf = nullptr, int *p_offset = nullptr) {
            uint32_t offset = UpperBound(0, cdf.size(), [=] AKR_XPU(int idx) { return cdf[idx] <= u; });
            if (p_offset) {
                *p_offset = offset;
            }
            Float du = u - cdf[offset];
            if ((cdf[offset + 1] - cdf[offset]) > 0)
                du /= (cdf[offset + 1] - cdf[offset]);
            if (pdf)
                *pdf = func[offset] / funcInt;
            return ((float)offset + du) / count();
        }

        [[nodiscard]] AKR_XPU size_t count() const { return func.size(); }
        [[nodiscard]] AKR_XPU Float integral() const { return funcInt; }

      private:
        Buffer<Float> func, cdf;
        Float funcInt;
    };
    AKR_VARIANT
    struct Distribution2D {
        astd::pmr::vector<Distribution1D> pConditionalV;
        astd::optional<Distribution1D> pMarginal;

      public:
        Distribution2D(MemoryResource *resource, const Float *data, size_t nu, size_t nv)
            : pConditionalV(astd::pmr::polymorphic_allocator<T>(resource)) {
            for (auto v = 0u; v < nv; v++) {
                pConditionalV.emplace_back(resource, &data[v * nu], nu);
            }
            std::vector<Float> m;
            for (auto v = 0u; v < nv; v++) {
                m.emplace_back(pConditionalV[v]->funcInt);
            }
            pMarginal.emplace(resource, &m[0], nv);
        }
        AKR_XPU Float2 sample_continuous(const Float2 &u, Float *pdf) const {
            int v;
            Float pdfs[2];
            auto d1 = pMarginal->sample_continuous(u[0], &pdfs[0], &v);
            auto d0 = pConditionalV[v].sample_continuous(u[1], &pdfs[1]);
            *pdf = pdfs[0] * pdfs[1];
            return vec2(d0, d1);
        }
        AKR_XPU Float pdf_continuous(const Float2 &p) const {
            auto iu = clamp<int>(p[0] * pConditionalV[0].count(), 0, pConditionalV[0].count() - 1);
            auto iv = clamp<int>(p[1] * pMarginal->count(), 0, pMarginal->count() - 1);
            return pConditionalV[iv].func[iu] / pMarginal->funcInt;
        }
    };
} // namespace akari