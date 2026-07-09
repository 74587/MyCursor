import React, { memo } from "react";
import type { AggregatedUsageData, ModelUsage } from "../types/usage";

interface AggregatedBreakdownProps {
  aggregatedUsage: AggregatedUsageData;
  className?: string;
}

const formatNumber = (num: string | number): string => {
  const numVal = typeof num === "string" ? parseInt(num) : num;
  if (isNaN(numVal)) return "0";
  return new Intl.NumberFormat().format(numVal);
};

const isRequestBased = (usage: AggregatedUsageData): boolean => {
  return usage.total_request_cost != null && usage.total_request_cost > 0;
};

const getBarColor = (index: number): string => {
  const colors = [
    "#6366f1", "#3b82f6", "#10b981", "#f59e0b",
    "#ec4899", "#8b5cf6", "#06b6d4", "#f97316",
  ];
  return colors[index % colors.length];
};

const ModelRow: React.FC<{
  model: ModelUsage;
  maxValue: number;
  index: number;
  requestBased: boolean;
}> = ({ model, maxValue, index, requestBased }) => {
  const value = requestBased
    ? (model.request_cost ?? 0)
    : model.total_cents;
  const ratio = maxValue > 0 ? (value / maxValue) * 100 : 0;
  const color = getBarColor(index);

  const displayValue = requestBased
    ? `${value} 次`
    : `$${(value / 100).toFixed(2)}`;

  const inputTokens = parseInt(model.input_tokens) || 0;
  const outputTokens = parseInt(model.output_tokens) || 0;
  const cacheReadTokens = parseInt(model.cache_read_tokens) || 0;

  return (
    <div className="py-2">
      <div className="flex items-center gap-3">
        <div className="flex-1 min-w-0">
          <div
            className="h-6 rounded-md flex items-center transition-all duration-500"
            style={{
              width: `${Math.max(ratio, 2)}%`,
              backgroundColor: color,
              opacity: 0.85,
            }}
          />
        </div>
        <div
          className="text-sm font-semibold w-20 text-right flex-shrink-0"
          style={{ color: 'var(--text-primary)' }}
        >
          {displayValue}
        </div>
        <div
          className="text-xs w-[220px] truncate flex-shrink-0"
          style={{ color: 'var(--text-secondary)' }}
          title={model.model_intent}
        >
          {model.model_intent}
        </div>
      </div>
      <div className="flex gap-3 mt-1 pl-0">
        <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
          入 {formatNumber(inputTokens)}
        </span>
        <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
          出 {formatNumber(outputTokens)}
        </span>
        {cacheReadTokens > 0 && (
          <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            缓存 {formatNumber(cacheReadTokens)}
          </span>
        )}
      </div>
    </div>
  );
};

export const AggregatedBreakdown: React.FC<AggregatedBreakdownProps> = memo(({
  aggregatedUsage,
  className = "",
}) => {
  const requestBased = isRequestBased(aggregatedUsage);

  const sorted = [...aggregatedUsage.aggregations].sort((a, b) => {
    const va = requestBased ? (a.request_cost ?? 0) : a.total_cents;
    const vb = requestBased ? (b.request_cost ?? 0) : b.total_cents;
    return vb - va;
  });

  const maxValue = sorted.length > 0
    ? (requestBased ? (sorted[0].request_cost ?? 0) : sorted[0].total_cents)
    : 0;

  const totalDisplay = requestBased
    ? `${aggregatedUsage.total_request_cost} 次`
    : `$${(aggregatedUsage.total_cost_cents / 100).toFixed(2)}`;

  return (
    <div
      className={`rounded-lg border p-4 ${className}`}
      style={{
        backgroundColor: 'var(--bg-primary)',
        borderColor: 'var(--border-primary)',
        transition: 'all 0.3s ease',
      }}
    >
      <div className="flex items-center justify-between mb-3">
        <h4
          className="text-md font-medium"
          style={{ color: 'var(--text-primary)' }}
        >
          消耗详情
        </h4>
        <span
          className="text-sm font-semibold"
          style={{ color: 'var(--primary-color)' }}
        >
          合计: {totalDisplay}
        </span>
      </div>

      {sorted.length === 0 ? (
        <div className="py-8 text-center text-sm" style={{ color: 'var(--text-tertiary)' }}>
          暂无模型用量数据
        </div>
      ) : (
        <div className="divide-y" style={{ borderColor: 'var(--border-primary)' }}>
          {sorted.map((model, index) => (
            <ModelRow
              key={model.model_intent}
              model={model}
              maxValue={maxValue}
              index={index}
              requestBased={requestBased}
            />
          ))}
        </div>
      )}
    </div>
  );
});

AggregatedBreakdown.displayName = "AggregatedBreakdown";
