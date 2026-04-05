import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  EmptyPlaceholder,
  Skeleton,
} from "@sensible-folio/ui";
import { useMemo } from "react";
import { DonutChart } from "./donut-chart";

interface FeeCategoryData {
  category: string;
  amount: number;
  percentage: number;
  transactions: number;
}

interface FeeCategoriesChartProps {
  feeCategories?: FeeCategoryData[];
  currency: string;
  isLoading?: boolean;
  onCategorySectionClick?: (categoryName: string) => void;
}

export const FeeCategoriesChart = ({
  feeCategories,
  currency,
  isLoading,
  onCategorySectionClick,
}: FeeCategoriesChartProps) => {
  const data = useMemo(() => {
    if (!feeCategories || feeCategories.length === 0) return [];

    return feeCategories
      .map((category) => ({
        name: category.category,
        value: category.amount,
        currency,
      }))
      .sort((a, b) => b.value - a.value);
  }, [feeCategories, currency]);

  if (isLoading) {
    return (
      <Card className="overflow-hidden backdrop-blur-sm">
        <CardHeader>
          <div className="flex items-center justify-between">
            <Skeleton className="h-5 w-[180px]" />
            <Skeleton className="h-5 w-[80px]" />
          </div>
        </CardHeader>
        <CardContent className="p-6">
          <div className="flex h-[200px] items-center justify-center">
            <Skeleton className="h-[150px] w-[150px] rounded-full" />
          </div>
        </CardContent>
      </Card>
    );
  }

  const handleInternalSectionClick = (sectionData: {
    name: string;
    value: number;
    currency: string;
  }) => {
    if (onCategorySectionClick) {
      onCategorySectionClick(sectionData.name);
    }
  };

  return (
    <Card className="overflow-hidden backdrop-blur-sm">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-muted-foreground text-sm font-medium uppercase tracking-wider">
            Fee Categories
          </CardTitle>
        </div>
      </CardHeader>
      <CardContent className="pt-0">
        {data.length > 0 ? (
          <DonutChart
            data={data}
            onPieEnter={() => {}}
            onSectionClick={handleInternalSectionClick}
            startAngle={180}
            endAngle={0}
            displayTooltip={false}
          />
        ) : (
          <EmptyPlaceholder
            description="There is no fee category data available."
            className="max-h-[160px]"
          />
        )}
      </CardContent>
    </Card>
  );
};
