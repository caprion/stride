/**
 * What this run meant, in a few sentences, straight under the map.
 *
 * The story is about the athlete against their own recent running: a jump
 * past their 30-day longest, a one-year best, more (or less) speed per
 * heartbeat than their usual run of that length, and the night before when
 * it plausibly explains a hard one. One chip says it in two or three words.
 */

import { useQuery } from '@tanstack/react-query'
import { Sparkles } from 'lucide-react'
import { api } from '../lib/api'
import { Badge, Card } from './ui'
import { chipTone } from './ActivityCard'

export function RunStoryCard({ activityId }: { activityId: number }) {
  const query = useQuery({
    queryKey: ['insights-activity', activityId],
    queryFn: () => api.insightsActivity(activityId),
    retry: false,
    staleTime: 60_000,
  })
  const story = query.data
  if (!story) return null
  return (
    <Card className="px-5 py-4">
      <div className="flex items-center gap-2">
        <Sparkles size={14} className="text-brand-600" />
        <p className="text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
          The story
        </p>
        <Badge tone={chipTone(story.tone)} className="ml-auto normal-case tracking-normal">
          {story.chip}
        </Badge>
      </div>
      <div className="mt-2 space-y-1.5">
        {story.story.map((line) => (
          <p key={line} className="text-[14px] leading-snug text-ink">
            {line}
          </p>
        ))}
      </div>
      {story.similar_runs > 0 ? (
        <p className="mt-2 text-[11px] text-ink-faint">
          Compared with {story.similar_runs} runs of a similar distance in the 90 days before.
        </p>
      ) : null}
    </Card>
  )
}
